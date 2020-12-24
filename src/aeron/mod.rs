use std::ffi::CString;

use aeron_rs::{
    example_config::{DEFAULT_MESSAGE_LENGTH, DEFAULT_STREAM_ID},
};

use crate::Arguments;

pub(crate) mod publisher;
pub(crate) mod subscriber;

#[derive(Clone)]
pub struct Settings {
    dir_prefix: String,
    port: i32,
    stream_id: i32,
    number_of_warmup_messages: i64,
    number_of_messages: i64,
    message_length: i32,
    linger_timeout_ms: u64,
}

impl Settings {
    pub fn new(args: Arguments) -> Self {
        Self {
            dir_prefix: String::new(),
            port: args.port,
            stream_id: DEFAULT_STREAM_ID.parse().unwrap(),
            number_of_warmup_messages: 0,
            number_of_messages: 10,
            message_length: DEFAULT_MESSAGE_LENGTH.parse().unwrap(),
            linger_timeout_ms: 100,
        }
    }
}

pub fn str_to_c(val: &str) -> CString {
    CString::new(val).expect("Error converting str to CString")
}
