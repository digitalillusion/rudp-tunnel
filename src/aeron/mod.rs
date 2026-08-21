use std::ffi::CString;

use aeron_rs::example_config::DEFAULT_STREAM_ID;

use crate::aeron::publisher::Publisher;
use crate::aeron::subscriber::Subscriber;
use crate::Arguments;
use aeron_rs::context::Context;
use aeron_rs::publication::Publication;
use aeron_rs::subscription::Subscription;
use std::sync::{Arc, Mutex};

pub(crate) mod publisher;
pub(crate) mod subscriber;

#[derive(Clone)]
pub struct Settings {
    dir_prefix: String,
    stream_id: i32,
    #[allow(dead_code)]
    number_of_warmup_messages: i64,
    #[allow(dead_code)]
    number_of_messages: i64,
    pub message_length: i32,
    #[allow(dead_code)]
    linger_timeout_ms: u64,
}

impl Settings {
    pub fn new(args: &Arguments) -> Self {
        Self {
            dir_prefix: args.dir_prefix.clone(),
            stream_id: DEFAULT_STREAM_ID.parse().unwrap(),
            number_of_warmup_messages: 0,
            number_of_messages: 10,
            message_length: args.mtu as i32,
            linger_timeout_ms: 100,
        }
    }
}

pub fn str_to_c(val: &str) -> CString {
    CString::new(val).expect("Error converting str to CString")
}

pub fn instance_publisher(
    context: Context,
    settings: &Settings,
    channel: &str,
) -> (Publisher, Arc<Mutex<Publication>>) {
    let publisher = Publisher::new(context, settings, channel)
        .unwrap_or_else(|_| panic!("Error creating publisher on channel {}", channel));
    let publication = publisher.publish();
    (publisher, publication)
}

pub fn instance_subscriber(
    context: Context,
    settings: &Settings,
    channel: &str,
) -> (Subscriber, Arc<Mutex<Subscription>>) {
    let subscriber = Subscriber::new(context, settings, channel)
        .unwrap_or_else(|_| panic!("Error creating subscriber on channel {}", channel));
    let subscription = subscriber.listen();
    (subscriber, subscription)
}
