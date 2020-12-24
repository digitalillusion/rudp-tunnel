mod aeron;
mod client;
mod server;

pub enum Mode {
    Client,
    Server,
}

#[derive(Clone)]
pub struct  Arguments {
    pub port: i32,
    pub channel: String,
    pub tunnel: String,
}

pub fn run(mode: Mode, args: Arguments) {
    match mode {
        Mode::Client => client::instance(args),
        Mode::Server => server::instance(args),
    }
}