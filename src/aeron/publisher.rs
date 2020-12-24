use std::{
    ffi::CString,
    sync::atomic::{AtomicBool, Ordering},
};

use aeron_rs::{
    aeron::Aeron,
    concurrent::{
        status::status_indicator_reader::channel_status_to_str,
    },
    context::Context,
    utils::errors::AeronError,
};
use lazy_static::lazy_static;
use aeron_rs::concurrent::atomic_buffer::{AtomicBuffer, AlignedBuffer};
use crate::aeron::{Settings, str_to_c};
use std::cell::{RefCell};
use std::sync::{Arc, Mutex};
use aeron_rs::publication::Publication;

lazy_static! {
    pub static ref RUNNING: AtomicBool = AtomicBool::from(true);
}

fn sig_int_handler() {
    RUNNING.store(false, Ordering::SeqCst);
}

fn error_handler(error: AeronError) {
    println!("Error: {:?}", error);
}

fn on_new_publication_handler(channel: CString, stream_id: i32, session_id: i32, correlation_id: i64) {
    println!(
        "Publication: {} {} {} {}",
        channel.to_str().unwrap(),
        stream_id,
        session_id,
        correlation_id
    );
}



pub struct Publisher {
    aeron: RefCell<Aeron>,
    settings: Settings,
}



impl Publisher {
    pub fn new(settings: Settings) -> Result<Self, Option<AeronError>> {
        pretty_env_logger::init();
        ctrlc::set_handler(move || {
            println!("received Ctrl+C!");
            sig_int_handler();
        })
            .expect("Error setting Ctrl-C handler");

        println!(
            "Publishing to channel {} on Stream ID {}",
            settings.channel, settings.stream_id
        );

        let mut context = Context::new();

        if !settings.dir_prefix.is_empty() {
            context.set_aeron_dir(settings.dir_prefix.clone());
        }

        println!("Using CnC file: {}", context.cnc_file_name());

        context.set_new_publication_handler(on_new_publication_handler);
        context.set_error_handler(error_handler);
        context.set_pre_touch_mapped_memory(true);

        let aeron = Aeron::new(context);

        if aeron.is_err() {
            return Err(aeron.err());
        }
        Ok(Self {
            aeron: RefCell::new(aeron.unwrap()),
            settings
        })
    }

    pub fn publish (self: &Self, buffer: [u8; 256], buffer_size: i32) {
        let publication = self.create_pubblication().unwrap();
        let channel_status = publication.lock().unwrap().channel_status();

        println!(
            "Publication channel status {}: {} ",
            channel_status,
            channel_status_to_str(channel_status)
        );

        let src_buffer = AtomicBuffer::from_aligned(&AlignedBuffer::with_capacity(256));
        src_buffer.put_bytes(0, &buffer);

        let result = publication.lock().unwrap().offer_part(src_buffer, 0, buffer_size);

        if let Ok(code) = result {
            println!("Sent with code {}!", code);
        } else {
            println!("Offer with error: {:?}", result.err());
        }

        if !publication.lock().unwrap().is_connected() {
            println!("No active subscribers detected");
        }
    }

    fn create_pubblication(self: &Self) -> Result<Arc<Mutex<Publication>>, AeronError> {
        let mut aeron = self.aeron.borrow_mut();
        // add the publication to start the process
        let publication_id = aeron
            .add_publication(str_to_c(&self.settings.channel), self.settings.stream_id)
            .expect("Error adding publication");

        let mut publication = aeron.find_publication(publication_id);
        while publication.is_err() {
            std::thread::yield_now();
            publication = aeron.find_publication(publication_id);
        };
        println!("Created pubblication {}", publication_id);

        publication
    }
}