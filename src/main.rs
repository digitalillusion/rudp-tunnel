use std::env;

use getopts::Options;
use log::info;

use rudp_tunnel::{Arguments, Mode, run};

fn main() {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"));

    if let Some((mode, args)) = parse_args() {
        run(mode, args);
    }
}

fn print_usage(program: &str, opts: Options) {
    info!("{}", opts.usage(&format!("Usage: {} [options]", program)));
}

fn parse_args() -> Option<(Mode, Arguments)> {
    let args: Vec<String> = env::args().collect();
    let program = &args[0];

    let mut opts = Options::new();
    opts.optflag("h", "help", "Show this usage message.");
    opts.optopt("p", "fport", "The port on which forward channel operates. Defaults to 40123", "FPORT");
    opts.optopt("q", "bport", "The port on which backward channel operates. Defaults to FPORT", "BPORT");
    opts.optopt("o", "origin", "Ip address to bind the client onto, origin of the tunnel. Mutually exclusive with -d ", "ORIGIN");
    opts.optopt("d", "destination", "Ip address where server sends packets, destination of the tunnel. Mutually exclusive with -o", "DESTINATION");
    opts.optopt("r", "remote", "Public network address of the remote server. Defaults to 0.0.0.0", "REMOTE");
    opts.optopt("i", "interface", "Routing interface. Defaults to 0.0.0.0", "INTERFACE");
    opts.optopt("f", "forward", "Forward channel, client to server.", "FORWARD");
    opts.optopt("b", "backward", "Backward channel, server to client.", "BACKWARD");

    match opts.parse(&args[1..]) {
        Ok(matches)  => {
            let is_client = matches.opt_present("origin");
            let is_server = matches.opt_present("destination");
            let fport = matches.opt_str("fport").map(|b| { b.parse::<i32>().unwrap() }).unwrap_or(40123);
            let bport = matches.opt_str("bport").map(|b| { b.parse::<i32>().unwrap() }).unwrap_or(fport);
            let remote = matches.opt_str("remote").unwrap_or(String::from("0.0.0.0"));
            let interface = matches.opt_str("interface");
            let interface= interface.map(|i| { format!("|interface={}", i) }).unwrap_or(String::new());
            let arguments = Arguments {
                fport: fport.to_owned(),
                bport: bport.to_owned(),
                remote: remote.to_owned(),
                sforward: matches.opt_str("forward").unwrap_or(String::from(format!("endpoint=0.0.0.0:{}{}", fport, interface))),
                sbackward: matches.opt_str("backward").unwrap_or(String::from(format!("endpoint={}:{}{}", remote, bport, interface))),
                cforward: matches.opt_str("forward").unwrap_or(String::from(format!("endpoint={}:{}{}", remote, fport, interface))),
                cbackward: matches.opt_str("backward").unwrap_or(String::from(format!("endpoint=0.0.0.0:{}{}", bport, interface))),
                destination: matches.opt_str("destination").unwrap_or(String::from(format!("0.0.0.0:0"))),
                origin: matches.opt_str("origin").unwrap_or(String::from(format!("0.0.0.0:0"))),
            };
            info!("{:?}", arguments);
            if is_client {
                Some((Mode::Client, arguments))
            } else if is_server {
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
