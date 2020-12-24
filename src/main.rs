use rudp_tunnel::{run, Mode, Arguments};

use getopts::Options;
use std::env;

fn main() {
    if let Some((mode, args)) = parse_args() {
        run(mode, args);
    }
}

fn print_usage(program: &str, opts: Options) {
    println!("{}", opts.usage(&format!("Usage: {} [options] -eENDPOINT -pPORT", program)));
}

fn parse_args() -> Option<(Mode, Arguments)> {
    let args: Vec<String> = env::args().collect();
    let program = &args[0];

    let mut opts = Options::new();
    opts.optflag("h", "help", "Show this usage message.");
    opts.optopt("p", "port", "Port to listen onto (client).", "PORT");
    opts.optopt("t", "tunnel", "Network address where to tunnel packets (server)", "TUNNEL");
    opts.optopt("c", "channel", "Aeron channel, defaults to 0.0.0.0:40123", "CHANNEL");

    match opts.parse(&args[1..]) {
        Ok(matches)  => {
            let arguments = Arguments {
                channel: matches.opt_str("channel").unwrap_or(String::from("localhost:40123")),
                tunnel: matches.opt_str("tunnel").unwrap_or(String::from("0.0.0.0:0")),
                port: matches.opt_str("port").unwrap_or(String::from("0")).parse().unwrap(),
            };
            if matches.opt_present("port") {
                Some((Mode::Client, arguments))
            } else if matches.opt_present("tunnel") {
                Some((Mode::Server, arguments))
            } else {
                print_usage(&program, opts);
                return None;
            }
        }
        Err(_) => { 
            print_usage(&program, opts);
            return None;
         }
    }
}
