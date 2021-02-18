use std::net::{UdpSocket};
use std::{slice, io};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use aeron_rs::concurrent::atomic_buffer::AtomicBuffer;
use aeron_rs::concurrent::logbuffer::header::Header;
use aeron_rs::utils::types::Index;
use log::{debug, error, info};

use crate::aeron::publisher::Publisher;
use crate::aeron::Settings;
use crate::aeron::subscriber::Subscriber;
use crate::Arguments;

pub struct Client {
    settings: Settings,
    channel_forward: String,
    channel_backward: String,
    endpoint: String,
}

impl Client {
    pub fn instance (args: Arguments) -> Self {
        Client {
            settings: Settings::new(args.to_owned()),
            channel_forward: format!("aeron:udp?{}", args.cforward),
            channel_backward: format!("aeron:udp?{}", args.cbackward),
            endpoint: args.endpoint,
        }
    }

    pub fn start (self, running: Arc<AtomicBool>) {
        let socket = UdpSocket::bind(self.endpoint.to_owned()).expect("Error binding socket input");
        socket.set_read_timeout(None).expect("Failed to set read timeout");
        socket.set_nonblocking(true).expect("Failed to enter non-blocking mode");
        let on_new_fragment = |buffer: &AtomicBuffer, offset: Index, length: Index, header: &Header| {
            let peer_addr = socket.peer_addr();
            if peer_addr.is_ok() {
                debug!("Sending {} bytes received on stream {} from session {} to endpoint {:?}", length, header.stream_id(), header.session_id(), peer_addr.unwrap());
                unsafe {
                    let slice_msg = slice::from_raw_parts_mut(buffer.buffer().offset(offset as isize), length as usize);
                    socket.send(slice_msg).unwrap_or_else(|e| {
                        error!("Can't tunnel packets to server: {}", e);
                        0
                    });
                }
            }
        };

        let subscriber_context = Subscriber::new_context(self.settings.clone());
        let subscriber = Subscriber::new(subscriber_context, self.settings.clone(), self.channel_backward)
            .expect("Error creating subscriber");
        let subscription = subscriber.listen();

        let publisher_context = Publisher::new_context(self.settings.clone());
        let publisher = Publisher::new(publisher_context, self.settings.clone(), self.channel_forward)
            .expect("Error creating publisher");
        let publication = publisher.publish();
        let stream_id = publication.lock().unwrap().stream_id();
        let session_id = publication.lock().unwrap().session_id();

        info!("Client listening to endpoint {} ", self.endpoint);

        while running.load(Ordering::SeqCst) {
            let mut recv_buff = vec![0; self.settings.message_length as usize];
            match socket.recv_from(&mut recv_buff) {
                Ok((n, addr)) => {
                    debug!("Publishing on stream {} from session {} {} bytes received from endpoint {:?}", stream_id, session_id, n, addr);
                    socket.connect(addr).expect("Error connecting socket output");
                    publisher.send(publication.to_owned(), &recv_buff, n as i32);
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

