mod aeron;
mod client;
mod server;

use crate::client::Client;
use crate::server::Server;

#[macro_use]
extern crate lazy_static;

pub enum Mode {
    Client,
    Server,
}

#[derive(Clone, Debug)]
pub struct Arguments {
    pub port: i32,
    pub server: String,
    pub sforward: String,
    pub cforward: String,
    pub cbackward: String,
    pub endpoint: String,
}

pub fn run(mode: Mode, args: Arguments) {
    match mode {
        Mode::Client => Client::instance(args).start(),
        Mode::Server => Server::instance(args).start(),
    }
}