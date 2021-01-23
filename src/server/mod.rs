use std::net::{UdpSocket, Ipv4Addr};
use std::slice;
use std::time::Duration;

use aeron_rs::concurrent::atomic_buffer::AtomicBuffer;
use aeron_rs::concurrent::logbuffer::header::Header;
use aeron_rs::utils::types::Index;
use log::{debug, info, error};

use crate::aeron::{Settings, subscriber};
use crate::Arguments;
use crate::aeron::publisher::Publisher;
use crate::aeron::subscriber::Subscriber;
use aeron_rs::publication::Publication;
use std::sync::{Mutex, Arc};
use std::cell::RefCell;
use lazy_static;
use std::rc::Rc;
use regex::Regex;
use std::str::FromStr;
use aeron_rs::image::Image;
use std::sync::atomic::{AtomicBool, Ordering};

lazy_static! {
    static ref NEW_CLIENT_IP_ADDRESSES: Mutex<Vec<String>> = Mutex::new(vec!());
}

type Publishers = Rc<RefCell<Vec<(Publisher, Arc<Mutex<Publication>>)>>>;

pub struct Server {
    port: i32,
    settings: Settings,
    publishers: Publishers,
    channel_forward: String,
    endpoint: String
}

impl Server {

    pub fn instance (args: Arguments) -> Self {
        Server {
            port: args.port,
            settings: Settings::new(args.clone()),
            publishers: Rc::new(RefCell::new(vec!())),
            channel_forward: format!("aeron:udp?{}", args.sforward),
            endpoint: args.endpoint,
        }
    }

    pub fn start(self, running: Arc<AtomicBool>) {
        let endpoint = self.endpoint.to_owned();
        let socket:UdpSocket = UdpSocket::bind("0.0.0.0:0").unwrap();
        socket.connect(endpoint.to_owned()).unwrap();
        socket.set_nonblocking(true).unwrap();

        let on_new_fragment = |buffer: &AtomicBuffer, offset: Index, length: Index, header: &Header| {
            debug!("{} bytes received on stream {} from session {} toward {}", length, header.stream_id(), header.session_id(), endpoint.to_owned());
            unsafe {
                let slice_msg = slice::from_raw_parts_mut(buffer.buffer().offset(offset as isize), length as usize);
                socket.send(slice_msg).unwrap_or_else(|e| {
                    error!("Can't tunnel packets to client: {}", e);
                    0
                });
            }
        };

        let mut subscriber_context = Subscriber::new_context(self.settings.clone());
        subscriber_context.set_available_image_handler(on_available_image);
        let subscriber = Subscriber::new(subscriber_context, self.settings.clone(), self.channel_forward.to_owned())
            .expect(format!("Error creating subscriber on channel {}", self.channel_forward).as_str());
        let subscription = subscriber.listen();

        info!("Server up and running, endpoint {}", self.endpoint);

        while running.load(Ordering::SeqCst) {
            let mut recv_buff = [0; 256];
            if let Ok((n, addr)) = socket.recv_from(&mut recv_buff) {
                debug!("{} bytes received from {:?}", n, addr);
                self.publishers.clone().borrow_mut().iter().for_each(|(publisher, publication)| {
                    publisher.send(publication.to_owned(), recv_buff, n as i32)
                });
            }

            let lock = NEW_CLIENT_IP_ADDRESSES.try_lock();
            if !lock.is_err() {
                lock.unwrap().drain(0..).for_each(|ip_address| {
                    let channel = format!("aeron:udp?endpoint={}:{}", ip_address, self.port);
                    add_publisher(self.publishers.clone(), self.settings.clone(), channel.to_string())
                });
            }

            subscriber.recv(subscription.to_owned(), &on_new_fragment);

            std::thread::sleep(Duration::from_millis(1));
        }
    }
}

fn add_publisher(publishers: Publishers, settings: Settings, channel: String) {
    let publisher_context = Subscriber::new_context(settings.clone());
    let publisher = Publisher::new(publisher_context, settings, channel.to_owned())
        .expect(format!("Error creating publisher on channel {}", channel).as_str());
    let publication = publisher.publish();
    publishers.borrow_mut().push((publisher, publication));
}

fn on_available_image(image: &Image)  {
    let source_identity = image.source_identity().to_str().unwrap().to_owned();
    subscriber::available_image_handler(image);

    let ipv4matcher = Regex::new(r"((?:[0-9]{1,3}\.){3}[0-9]{1,3}):(\d)").unwrap();
    let ip_adrr = ipv4matcher.captures_iter(source_identity.as_str()).next()
        .expect(format!("Cannot parse ip address from source identity {}", source_identity).as_str()).get(1)
        .map(|m| m.as_str())
        .unwrap();
    let ip_addr = Ipv4Addr::from_str(ip_adrr).unwrap();
    if !ip_addr.is_loopback() && !ip_addr.is_unspecified() {
        NEW_CLIENT_IP_ADDRESSES.lock().unwrap().push(ip_adrr.to_owned());
    }
}

