use std::ffi::CString;

use aeron_rs::{
    example_config::{DEFAULT_STREAM_ID},
};

use crate::Arguments;
use aeron_rs::context::Context;
use crate::aeron::publisher::Publisher;
use std::sync::{Arc, Mutex};
use aeron_rs::publication::Publication;
use crate::aeron::subscriber::Subscriber;
use aeron_rs::subscription::Subscription;

pub(crate) mod publisher;
pub(crate) mod subscriber;

#[derive(Clone)]
pub struct Settings {
    dir_prefix: String,
    stream_id: i32,
    number_of_warmup_messages: i64,
    number_of_messages: i64,
    pub message_length: i32,
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

pub fn instance_publisher(context: Context, settings: &Settings, channel: &String) -> (Publisher, Arc<Mutex<Publication>>) {
    let publisher = Publisher::new(context, settings, channel)
        .expect(format!("Error creating publisher on channel {}", channel).as_str());
    let publication = publisher.publish();
    (publisher, publication)
}

pub fn instance_subscriber(context: Context, settings: &Settings, channel: &String) -> (Subscriber, Arc<Mutex<Subscription>>) {
    let subscriber = Subscriber::new(context, settings, channel)
        .expect(format!("Error creating subscriber on channel {}", channel).as_str());
    let subscription = subscriber.listen();
    (subscriber, subscription)
}