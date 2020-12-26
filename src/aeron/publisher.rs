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
    utils::errors::AeronError,
};
use aeron_rs::concurrent::atomic_buffer::{AlignedBuffer, AtomicBuffer};
use aeron_rs::publication::Publication;

use crate::aeron::{Settings, str_to_c};

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
    channel: String,
}

impl Publisher {
    pub fn new(settings: Settings, channel: String) -> Result<Self, Option<AeronError>> {
        println!("Instance Publisher at {} on Stream ID {}", channel, settings.stream_id);

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
            settings,
            channel,
        })
    }

    pub fn publish (self: &Self) -> Arc<Mutex<Publication>> {
        let publication = self.create_pubblication().expect("Error creating publication");
        let channel_status = publication.lock().unwrap().channel_status();

        println!(
            "Publication channel status {}: {} ",
            channel_status,
            channel_status_to_str(channel_status)
        );

        publication
    }

    pub fn send(self: &Self, publication: Arc<Mutex<Publication>>, buffer: [u8; 256], buffer_size: i32) {
        let aligned_buffer = AlignedBuffer::with_capacity(256);
        let src_buffer = AtomicBuffer::from_aligned(&aligned_buffer);
        src_buffer.put_bytes(0, &buffer);

        let result = publication.lock().unwrap().offer_part(src_buffer, 0, buffer_size);

        if let Err(error) = result {
            println!("Offer with error: {:?}", error);
        }

        if !publication.lock().unwrap().is_connected() {
            println!("No active subscribers detected");
        }
    }

    fn create_pubblication(self: &Self) -> Result<Arc<Mutex<Publication>>, AeronError> {
        let mut aeron = self.aeron.borrow_mut();
        // add the publication to start the process
        let publication_id = aeron
            .add_publication(str_to_c(&self.channel), self.settings.stream_id)
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