
use std::net::UdpSocket;
use std::{slice, io};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use aeron_rs::concurrent::atomic_buffer::AtomicBuffer;
use aeron_rs::concurrent::logbuffer::header::Header;
use aeron_rs::utils::types::Index;
use log::{debug, error, info};

use crate::aeron::Settings;
use crate::aeron::publisher::Publisher;
use crate::aeron::subscriber::Subscriber;
use crate::Arguments;

pub struct Server {
    settings: Settings,
    channel_forward: String,
    channel_backward: String,
    endpoint: String
}

impl Server {

    pub fn instance (args: Arguments) -> Self {
        Server {
            settings: Settings::new(args.clone()),
            channel_forward: format!("aeron:udp?{}", args.sforward),
            channel_backward: format!("aeron:udp?{}", args.sbackward),
            endpoint: args.endpoint,
        }
    }

    pub fn start(self, running: Arc<AtomicBool>) {
        let endpoint = self.endpoint.to_owned();
        let socket:UdpSocket = UdpSocket::bind("0.0.0.0:0").unwrap();
        socket.set_read_timeout(None).expect("Failed to set read timeout");
        socket.connect(endpoint.to_owned()).expect("Failed to connect to endpoint");
        socket.set_nonblocking(true).expect("Failed to enter non-blocking mode");

        let subscriber_context = Subscriber::new_context(self.settings.clone());
        let subscriber = Subscriber::new(subscriber_context, self.settings.clone(), self.channel_forward.to_owned())
            .expect(format!("Error creating subscriber on channel {}", self.channel_forward).as_str());
        let subscription = subscriber.listen();

        let publisher_context = Publisher::new_context(self.settings.clone());
        let publisher = Publisher::new(publisher_context, self.settings.clone(), self.channel_backward)
            .expect("Error creating publisher");
        let publication = publisher.publish();
        let stream_id = publication.lock().unwrap().stream_id();
        let session_id = publication.lock().unwrap().session_id();

        info!("Server up and running, endpoint {}", self.endpoint);

        let on_new_fragment = |buffer: &AtomicBuffer, offset: Index, length: Index, header: &Header| {
            debug!("Sending {} bytes received on stream {} from session {} to endpoint {}", length, header.stream_id(), header.session_id(), socket.peer_addr().unwrap());
            unsafe {
                let slice_msg = slice::from_raw_parts_mut(buffer.buffer().offset(offset as isize), length as usize);
                socket.send(slice_msg).unwrap_or_else(|e| {
                    error!("Can't send packets to endpoint: {}", e);
                    0
                });
                publisher.send(publication.to_owned(), slice_msg, length);
            }
        };

        while running.load(Ordering::SeqCst) {
            let mut recv_buff = vec![0; self.settings.message_length as usize];
            match socket.recv_from(&mut recv_buff) {
                Ok((n, addr)) => {
                    debug!("Publishing on stream {} from session {} {} bytes received from endpoint {:?}", stream_id, session_id, n, addr);
                    publisher.send(publication.to_owned(), &recv_buff, n as i32)
                }
                Err(err) => {
                    if err.kind() != io::ErrorKind::WouldBlock {
                        error!("Error receiving from endpoint {:?}", err)
                    }
                }
            }

            subscriber.recv(subscription.to_owned(), &on_new_fragment);

            std::thread::sleep(Duration::from_millis(1));
        }
    }
}
