use crate::aeron::publisher::Publisher;
use std::sync::{Arc, Mutex};
use aeron_rs::publication::Publication;
use crate::aeron::subscriber::Subscriber;
use aeron_rs::subscription::Subscription;
use std::time::{SystemTime, Duration};
use crate::aeron::{Settings, subscriber, instance_subscriber, instance_publisher};
use aeron_rs::image::Image;

use log::{debug};

use super::{DISCONNECTED_SESSIONS, CONNECTED_SESSIONS};
use std::net::SocketAddr;
use aeron_rs::concurrent::atomic_buffer::AtomicBuffer;
use aeron_rs::utils::types::Index;
use aeron_rs::concurrent::logbuffer::header::Header;
use std::ops::Add;
use crate::Timeout;

#[derive(Clone)]
pub struct ClientSlot {
    stream_id: i32,
    publisher_session_id: i32,
    subscriber_session_id: Arc<Mutex<i32>>,
    port: usize,
    control: usize,
    publisher: Arc<Publisher>,
    publication: Arc<Mutex<Publication>>,
    subscriber: Arc<Subscriber>,
    subscription: Arc<Mutex<Subscription>>,
    timeout: Arc<Mutex<SystemTime>>,
    closed: Arc<Mutex<bool>>,
}

impl ClientSlot {

    pub fn new(settings: &Settings, channel_forward: String, channel_backward: String, port: usize, control: usize) -> Self {
        let mut subscriber_context = Subscriber::new_context(settings);
        subscriber_context.set_unavailable_image_handler(on_unavailable_image);
        subscriber_context.set_available_image_handler(on_available_image);
        let (client_subscriber, client_subscription) =
            instance_subscriber(subscriber_context,  settings, &channel_forward);

        let publisher_context = Publisher::new_context(settings);
        let (client_publisher, client_publication) =
            instance_publisher(publisher_context, settings, &channel_backward);
        let stream_id = client_publication.lock().unwrap().stream_id();
        ClientSlot {
            stream_id,
            subscriber_session_id: Arc::new(Mutex::new(-1)),
            publisher_session_id: client_publication.clone().lock().unwrap().session_id(),
            port,
            control,
            publisher: Arc::new(client_publisher),
            publication: client_publication,
            subscriber: Arc::new(client_subscriber),
            subscription: client_subscription,
            timeout: Arc::new(Mutex::new(SystemTime::now().add(Duration::from_secs(Timeout::CONNECTION_SECONDS)))),
            closed: Arc::new(Mutex::new(false))
        }
    }

    pub fn is_publishing_on_session(&self, session_id: i32) -> bool {
        self.publisher_session_id == session_id
    }

    pub fn has_subscribers_on_session(&self, session_id: i32) -> bool {
        *self.subscriber_session_id.lock().unwrap() == session_id || self.subscription.lock().unwrap().image_by_session_id(session_id).is_some()
    }

    pub fn activate(&self, session_id: i32) {
        *self.subscriber_session_id.lock().unwrap() = session_id;
        *self.timeout.lock().unwrap() = SystemTime::now().checked_add(Duration::from_secs(Timeout::SESSION_SECONDS)).unwrap();
    }

    pub fn is_timeout_elapsed(&self) -> bool {
        *self.timeout.lock().unwrap() <= SystemTime::now()
    }

    pub fn publish(&self, slice_msg: &mut [u8], slice_size: usize, origin: SocketAddr) {
        *self.timeout.lock().unwrap() = SystemTime::now().add(Duration::from_secs(Timeout::SESSION_SECONDS));
        debug!("Publishing on stream {} from session {} {} bytes received from endpoint {:?}", self.stream_id, self.publisher_session_id, slice_size, origin);
        self.publisher.send(self.publication.to_owned(), slice_msg, slice_size)
    }

    pub fn receive<F>(&self, on_new_fragment: F)
        where F: Fn(&AtomicBuffer, Index, Index, &Header) -> () {
        *self.timeout.lock().unwrap() = SystemTime::now().add(Duration::from_secs(Timeout::SESSION_SECONDS));
        self.subscriber.recv(self.subscription.to_owned(), on_new_fragment);
    }

    pub fn close(&self) {
        *self.closed.lock().unwrap() = true;
        self.publication.lock().unwrap().close();
        self.subscription.lock().unwrap().close_and_remove_images();
    }

    pub fn is_closed(&self) -> bool {
        *self.closed.lock().unwrap()
    }

}

fn on_unavailable_image(image: &Image)  {
    subscriber::unavailable_image_handler(image);
    DISCONNECTED_SESSIONS.lock().unwrap().push(image.session_id());
}

fn on_available_image(image: &Image)  {
    subscriber::available_image_handler(image);
    CONNECTED_SESSIONS.lock().unwrap().push(image.session_id());
}
