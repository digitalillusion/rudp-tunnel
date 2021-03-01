mod client_slot;

use std::net::SocketAddr;
use std::{slice, io};
use std::sync::{Mutex, Arc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration};
use std::cell::RefCell;

use aeron_rs::concurrent::atomic_buffer::AtomicBuffer;
use aeron_rs::concurrent::logbuffer::header::Header;
use aeron_rs::utils::types::Index;
use log::{debug, error, info};

use crate::aeron::{Settings, instance_subscriber, instance_publisher};
use crate::aeron::publisher::Publisher;
use crate::aeron::subscriber::Subscriber;
use crate::{Arguments, attach_endpoint};

use lazy_static;
use crate::messages::{HandshakeRequest, HandshakeResponse, Failure, FailureDetails};
use crate::server::client_slot::ClientSlot;
use std::str::FromStr;

lazy_static! {
    pub static ref CONNECTED_SESSIONS: Mutex<Vec<i32>> = Mutex::new(vec!());
    pub static ref DISCONNECTED_SESSIONS: Mutex<Vec<i32>> = Mutex::new(vec!());
}

pub struct Server {
    settings: Settings,
    channel_forward: String,
    channel_backward: String,
    args: Arguments,
    slots: RefCell<Vec<Option<ClientSlot>>>
}

impl Server {

    pub fn instance (args: &Arguments) -> Self {
        Server {
            settings: Settings::new(args),
            channel_forward: format!("aeron:udp?{}", args.sforward),
            channel_backward: format!("aeron:udp?{}", args.sbackward),
            args: args.clone(),
            slots: RefCell::new(vec![ None; args.max_clients])
        }
    }

    pub fn start(&self, running: Arc<AtomicBool>) {
        let socket = attach_endpoint(&self.args);

        let (subscriber, subscription) =
            instance_subscriber(Subscriber::new_context(&self.settings), &self.settings, &self.channel_forward);
        let (publisher, publication) =
            instance_publisher(Publisher::new_context(&self.settings), &self.settings, &self.channel_backward);

        let on_client_handshake = |buffer: &AtomicBuffer, offset: Index, length: Index, header: &Header| {
            debug!("Received handshake request from session_id={} stream_id={} (length={})", header.session_id(), header.stream_id(), length);
            let request= unsafe {
                let slice_msg = slice::from_raw_parts_mut(buffer.buffer().offset(offset as isize), length as usize);
                bincode::deserialize(&slice_msg).unwrap()
            };

            let failure_details = FailureDetails { session_id: header.session_id() };
            let position = self.slots.borrow().iter()
                .position(|s| s.is_some() && s.clone().unwrap().is_publishing_on_session(header.session_id()))
                .or_else(|| self.slots.borrow().iter().position(|s| s.is_none()));
            let result = match position {
                Some(index) => self.handshake(header, request, failure_details, index),
                None => Err(Failure::HandshakeFailedServerFull(failure_details))
            };
            let response = bincode::serialize(&result).unwrap();
            debug!("Sending handshake response (success={}, length={})", result.is_ok(), response.len());
            publisher.send(publication.to_owned(), response.as_ref(), response.len());
        };


        let on_subscriber_receive = |buffer: &AtomicBuffer, offset: Index, length: Index, header: &Header| {
            let peer_addr = socket.peer_addr().unwrap_or(SocketAddr::from_str("0.0.0.0:0").unwrap());
            debug!("Sending {} bytes received on stream {} from session {} to endpoint {}", length, header.stream_id(), header.session_id(), peer_addr.clone());
            unsafe {
                let slice_msg = slice::from_raw_parts_mut(buffer.buffer().offset(offset as isize), length as usize);
                if socket.peer_addr().is_ok() {
                    socket.send(slice_msg).unwrap_or_else(|e| {
                        error!("Can't send packets to endpoint: {}", e);
                        0
                    });
                }
                self.slots.borrow().iter().enumerate()
                    .filter(|(_, slot)| slot.is_some() && !slot.as_ref().clone().unwrap().has_subscribers_on_session(header.session_id()))
                    .for_each(|(index, slot)| {
                        debug!("Forwarding {} bytes to subscriber on slot {}/{}", length, index + 1, self.args.max_clients);
                        slot.as_ref().unwrap().publish(slice_msg, length as usize, peer_addr)
                    });
            }
        };


        info!("Server waiting for handshakes, {} to endpoint {}", if self.args.listen { "listening" } else { "connected" }, self.args.endpoint);

        while running.load(Ordering::SeqCst) {
            let mut recv_buff = vec![0; self.settings.message_length as usize];
            match socket.recv_from(&mut recv_buff) {
                Ok((n, addr)) => {
                    self.slots.borrow().iter()
                        .filter(|slot| slot.is_some())
                        .for_each(|slot| slot.as_ref().clone().unwrap().publish(recv_buff.as_mut_slice(), n, addr));
                }
                Err(err) => {
                    if err.kind() != io::ErrorKind::WouldBlock {
                        error!("Error receiving from endpoint {:?}", err)
                    }
                }
            }

            self.slots.borrow().iter()
                .filter(|slot| slot.is_some())
                .for_each(|slot| slot.clone().unwrap().receive(&on_subscriber_receive));

            self.handle_disconnections();

            self.handle_connections();
            subscriber.recv(subscription.to_owned(), &on_client_handshake);

            std::thread::sleep(Duration::from_millis(1));
        }
    }

    fn handle_connections(&self) {
        let lock = CONNECTED_SESSIONS.try_lock();
        if !lock.is_err() {
            lock.unwrap().drain(0..).for_each(|session_id| {
                let slots = self.slots.borrow();
                if let Some((position, slot)) = slots.iter()
                    .enumerate()
                    .find(|(_, slot)| {
                        slot.is_some() && slot.as_ref().clone().unwrap().has_subscribers_on_session(session_id)
                    }) {
                    slot.as_ref().unwrap().activate(session_id);
                    info!("ClientSlot at position {}/{} is now on an active session_id={}", position + 1, self.args.max_clients, session_id);
                }
            });
        }
    }

    fn handle_disconnections(&self) {
        let lock = DISCONNECTED_SESSIONS.try_lock();
        if !lock.is_err() {
            lock.unwrap().drain(0..).for_each(|session_id| {
                let mut slots = self.slots.borrow_mut();
                if let Some(position) = slots.iter()
                    .position(|slot| {
                        if let Some(slot) = slot {
                            !slot.is_closed() &&
                            (slot.is_publishing_on_session(session_id) || slot.has_subscribers_on_session(session_id))
                        } else {
                            false
                        }
                    }) {
                    let slot = slots[position].take();
                    slot.unwrap().close();
                    info!("ClientSlot at position {}/{} is now free since associated session is closed", position + 1, self.args.max_clients)
                }
            });
            if let Some(position) = self.slots.borrow().iter()
                .position(|slot| {
                    slot.is_some() && slot.as_ref().unwrap().is_timeout_elapsed()
                }) {
                let slot = self.slots.borrow_mut()[position].take();
                slot.unwrap().close();
                info!("ClientSlot at position {}/{} is now free since associated session timed out", position + 1, self.args.max_clients);
            }
        }
    }

    fn handshake(&self, header: &Header, request: HandshakeRequest, failure_details: FailureDetails, index: usize) -> Result<HandshakeResponse, Failure> {
        let slot_index = index + 1;
        let port = self.args.port + slot_index;
        let control = self.args.control + slot_index;

        let channel_forward = format!("aeron:udp?endpoint=0.0.0.0:{}{}", port, self.args.interface);
        let channel_backward = format!("aeron:udp?endpoint={}:{}{}|control={}:{}|control-mode=dynamic", self.args.public, port, self.args.interface, self.args.public, control);
        let client_slot = ClientSlot::new(&self.settings, channel_forward, channel_backward, port, control);

        let encrypted_session_id = header.session_id().wrapping_mul(request.key);
        debug!("Computing verification: {} * {} = {}", header.session_id(), request.key, encrypted_session_id);

        let handshake_response = HandshakeResponse {
            verification: encrypted_session_id,
            port,
            control
        };
        info!("Client handshake on slot {}/{}, sending {:?}", slot_index, self.args.max_clients, handshake_response);

        if let None = self.slots.borrow_mut()[index].replace(client_slot) {
            Ok(handshake_response)
        } else {
            Err(Failure::HandshakeFailedTooManyConnections(failure_details))
        }
    }
}