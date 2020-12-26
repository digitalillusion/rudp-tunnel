use std::{
    ffi::CString,
};
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

use aeron_rs::{
    aeron::Aeron,
    concurrent::{
        status::status_indicator_reader::channel_status_to_str,
    },
    context::Context,
    image::Image,
    utils::{errors::AeronError},
};
use aeron_rs::concurrent::atomic_buffer::AtomicBuffer;
use aeron_rs::concurrent::logbuffer::header::Header;
use aeron_rs::subscription::Subscription;

use crate::aeron::{Settings, str_to_c};

fn on_new_subscription_handler(channel: CString, stream_id: i32, correlation_id: i64) {
    println!("Subscription: {} {} {}", channel.to_str().unwrap(), stream_id, correlation_id);
}

fn available_image_handler(image: &Image) {
    println!(
        "Available image correlation_id={} session_id={} at position={} from {}",
        image.correlation_id(),
        image.session_id(),
        image.position(),
        image.source_identity().to_str().unwrap()
    );
}

fn unavailable_image_handler(image: &Image) {
    println!(
        "Unavailable image correlation_id={} session_id={} at position={} from {}",
        image.correlation_id(),
        image.session_id(),
        image.position(),
        image.source_identity().to_str().unwrap()
    );
}

fn error_handler(error: AeronError) {
    println!("Error: {:?}", error);
}

pub struct Subscriber {
    aeron: RefCell<Aeron>,
    settings: Settings,
    channel: String,
}

impl Subscriber {
    pub fn new(settings: Settings, channel: String) -> Result<Self, Option<AeronError>> {
        println!("Instance Subscriber at {} on Stream ID {}", channel, settings.stream_id);

        let mut context = Context::new();

        if !settings.dir_prefix.is_empty() {
            context.set_aeron_dir(settings.dir_prefix.clone());
        }

        println!("Using CnC file: {}", context.cnc_file_name());

        context.set_new_subscription_handler(on_new_subscription_handler);
        context.set_available_image_handler(available_image_handler);
        context.set_unavailable_image_handler(unavailable_image_handler);
        context.set_error_handler(error_handler);
        context.set_pre_touch_mapped_memory(true);

        let aeron = Aeron::new(context);

        if aeron.is_err() {
            return Err(aeron.err());
        }
        Ok(Self {
            aeron: RefCell::new(aeron.unwrap()),
            settings,
            channel
        })
    }

    pub fn listen(self: &Self) -> Arc<Mutex<Subscription>> {
        let subscription = self.create_subscription().expect("Error creating subscription");
        let channel_status = subscription.lock().unwrap().channel_status();

        println!(
            "Subscription channel status {}: {}",
            channel_status,
            channel_status_to_str(channel_status)
        );

        subscription
    }

    pub fn recv(self: &Self, subscription: Arc<Mutex<Subscription>>, mut on_new_fragment: &dyn Fn(&AtomicBuffer, i32, i32, &Header) -> ()) {
        subscription.lock().unwrap().poll(&mut on_new_fragment, 10);
    }

    fn create_subscription(self: &Self) -> Result<Arc<Mutex<Subscription>>, AeronError> {
        let mut aeron = self.aeron.borrow_mut();
        let subscription_id = aeron
            .add_subscription(str_to_c(&self.channel), self.settings.stream_id)
            .expect("Error adding subscription");

        let mut subscription = aeron.find_subscription(subscription_id);
        while subscription.is_err() {
            std::thread::yield_now();
            subscription = aeron.find_subscription(subscription_id);
        }
        println!("Created new subscription {}", subscription_id);
        subscription
    }
}