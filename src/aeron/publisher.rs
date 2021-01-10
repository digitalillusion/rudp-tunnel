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
use log::{error, info};

use crate::aeron::{Settings, str_to_c};

pub fn error_handler(error: AeronError) {
    error!("Error: {:?}", error);
}

pub fn on_new_publication_handler(channel: CString, stream_id: i32, session_id: i32, correlation_id: i64) {
    info!(
        "Publication: {} (stream={} session={} correlation={})",
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

    pub fn new_context(settings: Settings) -> Context {
        let mut context = Context::new();

        if !settings.dir_prefix.is_empty() {
            context.set_aeron_dir(settings.dir_prefix.clone());
        }

        info!("Using CnC file: {}", context.cnc_file_name());

        context.set_new_publication_handler(on_new_publication_handler);
        context.set_error_handler(error_handler);
        context.set_pre_touch_mapped_memory(true);

        context
    }

    pub fn new(context: Context, settings: Settings, channel: String) -> Result<Self, Option<AeronError>> {
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

        if publication.lock().is_err() {
            info!(
                "Publication channel status {}: {}, {:?}",
                channel_status,
                channel_status_to_str(channel_status),
                publication.lock().err()
            );
        }

        publication
    }

    pub fn send(self: &Self, publication: Arc<Mutex<Publication>>, buffer: [u8; 256], buffer_size: i32) {
        let aligned_buffer = AlignedBuffer::with_capacity(256);
        let src_buffer = AtomicBuffer::from_aligned(&aligned_buffer);
        src_buffer.put_bytes(0, &buffer);

        let result = publication.lock().unwrap().offer_part(src_buffer, 0, buffer_size);

        if let Err(error) = result {
            error!("Send error: {:?}", error);
        }

        if !publication.lock().unwrap().is_connected() {
            error!("No active subscribers detected on channel {}", self.channel);
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
        info!("Created pubblication {}", publication_id);

        publication
    }
}