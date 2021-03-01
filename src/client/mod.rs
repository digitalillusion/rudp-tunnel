use std::net::SocketAddr;
use std::{slice, io};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime};

use aeron_rs::concurrent::atomic_buffer::AtomicBuffer;
use aeron_rs::concurrent::logbuffer::header::Header;
use aeron_rs::utils::types::Index;
use log::{debug, error, info};

use crate::aeron::publisher::Publisher;
use crate::aeron::{Settings, instance_subscriber, instance_publisher};
use crate::aeron::subscriber::Subscriber;
use crate::{Arguments, Timeout, attach_endpoint};
use crate::messages::{HandshakeRequest, HandshakeResponse, Failure};
use std::cell::RefCell;
use std::ops::Add;
use std::str::FromStr;

pub struct Client {
    settings: Settings,
    channel_forward: String,
    channel_backward: String,
    args: Arguments,
}

impl Client {
    pub fn instance (args: &Arguments) -> Self {
        Client {
            settings: Settings::new(args),
            channel_forward: format!("aeron:udp?{}", args.cforward),
            channel_backward: format!("aeron:udp?{}", args.cbackward),
            args: args.clone(),
        }
    }

    pub fn start (self, running: Arc<AtomicBool>) {
        match self.handshake(&running) {
            Ok(connection) => {
                info!("Connection parameters: {:?}", connection);

                let channel_forward = format!("aeron:udp?endpoint={}:{}{}", self.args.server, connection.port, self.args.interface);
                let channel_backward = format!("aeron:udp?endpoint=0.0.0.0:0{}|control={}:{}|control-mode=dynamic", self.args.interface, self.args.server, connection.control);

                let subscriber_context = Subscriber::new_context(&self.settings);
                let (subscriber, subscription) =
                    instance_subscriber(subscriber_context,  &self.settings, &channel_backward);

                let publisher_context = Publisher::new_context( &self.settings);
                let (publisher, publication) =
                    instance_publisher(publisher_context, &self.settings, &channel_forward);
                let stream_id = publication.lock().unwrap().stream_id();
                let session_id = publication.lock().unwrap().session_id();

                let socket = attach_endpoint(&self.args);
                let on_new_fragment = |buffer: &AtomicBuffer, offset: Index, length: Index, header: &Header| {
                    let peer_addr = socket.peer_addr().unwrap_or(SocketAddr::from_str("0.0.0.0:0").unwrap());
                    debug!("Sending {} bytes received on stream {} from session {} to endpoint {:?}", length, header.stream_id(), header.session_id(), peer_addr);
                    if socket.peer_addr().is_ok() {
                        unsafe {
                            let slice_msg = slice::from_raw_parts_mut(buffer.buffer().offset(offset as isize), length as usize);
                            socket.send(slice_msg).unwrap_or_else(|e| {
                                error!("Can't tunnel packets to server: {}", e);
                                0
                            });
                        }
                    }
                };

                info!("Client {} to endpoint {} ", if self.args.listen { "listening" } else { "connected" }, self.args.endpoint);

                while running.load(Ordering::SeqCst) {
                    let mut recv_buff = vec![0; self.settings.message_length as usize];
                    match socket.recv_from(&mut recv_buff) {
                        Ok((n, addr)) => {
                            debug!("Publishing on stream {} from session {} {} bytes received from endpoint {:?}", stream_id, session_id, n, addr);
                            socket.connect(addr).expect("Error connecting socket output");
                            publisher.send(publication.to_owned(), &recv_buff, n);
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
            Err(failure) => error!("Handshake failed: {:?}", failure)
        }
    }

    fn handshake(&self, running: &Arc<AtomicBool>) -> Result<HandshakeResponse, Failure> {
        let handshake_request = HandshakeRequest::new();
        info!("Starting handshake: {:?}", handshake_request);

        let subscriber_context = Subscriber::new_context(&self.settings);
        let (subscriber, subscription) =
            instance_subscriber(subscriber_context, &self.settings, &self.channel_backward);

        let publisher_context = Publisher::new_context(&self.settings);
        let (publisher, publication) =
            instance_publisher(publisher_context, &self.settings, &self.channel_forward);
        let stream_id = publication.lock().unwrap().session_id();
        let session_id = publication.lock().unwrap().session_id();

        let mut handshake_request_timeout = SystemTime::now();
        let handshake_response: RefCell<Option<Result<HandshakeResponse, Failure>>> = RefCell::new(None);
        let on_handshake_response = |buffer: &AtomicBuffer, offset: Index, length: Index, header: &Header| {
            unsafe {
                let slice_msg = slice::from_raw_parts_mut(buffer.buffer().offset(offset as isize), length as usize);
                debug!("Received handshake response from stream_id={} session_id={} (length={})", header.stream_id(), header.session_id(), length);
                let deserialized: Result<HandshakeResponse, Failure> = bincode::deserialize(slice_msg).unwrap();
                let encrypted_session_id = session_id.wrapping_mul(handshake_request.key);
                debug!("Computing verification: {} * {} = {}", session_id, handshake_request.key, encrypted_session_id);
                match deserialized {
                    Ok(response) => {
                        if response.verification == encrypted_session_id {
                            handshake_response.replace(Some(deserialized.clone()));
                        } else {
                            debug!("Ignoring handshake success, verification mismatch (local={}, received={})", encrypted_session_id, response.verification);
                        }
                    },
                    Err(failure) => {
                        match failure {
                            Failure::HandshakeFailedServerFull(failure_details) |
                            Failure::HandshakeFailedTooManyConnections(failure_details) => {
                                if failure_details.session_id == encrypted_session_id {
                                    handshake_response.replace(Some(deserialized.clone()));
                                } else {
                                    debug!("Ignoring handshake failure, verification mismatch (local={}, received={})", encrypted_session_id, failure_details.session_id)
                                }
                            }
                        }
                    }
                }
            }
        };


        while running.load(Ordering::SeqCst) && handshake_response.borrow().is_none() {
            if handshake_request_timeout < SystemTime::now() {
                handshake_request_timeout = handshake_request_timeout.add(Duration::from_secs(Timeout::HANDSHAKE_RETRY_SECONDS));
                let message = bincode::serialize(&handshake_request).unwrap();
                info!("Sending handshake request on stream_id={} session_id={} (length={})", stream_id, session_id, message.len());
                publisher.send(publication.to_owned(), message.as_ref(), message.len());
            }
            subscriber.recv(subscription.to_owned(), &on_handshake_response);
            std::thread::sleep(Duration::from_millis(1));
        }

        subscription.lock().unwrap().close_and_remove_images();
        publication.lock().unwrap().close();
        let result = handshake_response.borrow().unwrap();
        result
    }
}

