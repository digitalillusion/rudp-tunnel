mod aeron;
mod client;
mod server;

pub enum Mode {
    Client,
    Server,
}

#[derive(Clone, Debug)]
pub struct  Arguments {
    pub origin: String,
    pub fport: i32,
    pub bport: i32,
    pub remote: String,
    pub sforward: String,
    pub sbackward: String,
    pub cforward: String,
    pub cbackward: String,
    pub destination: String,
}

pub fn run(mode: Mode, args: Arguments) {
    match mode {
        Mode::Client => client::instance(args),
        Mode::Server => server::instance(args),
    }
}