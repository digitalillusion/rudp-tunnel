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
    opts.optopt("p", "port", "The port on which tunnel operates. Defaults to 40123", "PORT");
    opts.optopt("e", "endpoint", "Network address where to send packets, endpoint of the tunnel.", "ENDPOINT");
    opts.optopt("s", "server", "Public Ip address of the server. Defaults to 0.0.0.0", "SERVER");
    opts.optopt("i", "interface", "Routing interface. Defaults to 0.0.0.0", "INTERFACE");

    match opts.parse(&args[1..]) {
        Ok(matches)  => {
            let is_server = !matches.opt_present("server");
            let port = matches.opt_str("port").map(|b| { b.parse::<i32>().unwrap() }).unwrap_or(40123);
            let server = matches.opt_str("server").unwrap_or(String::from("0.0.0.0"));
            let interface = matches.opt_str("interface");
            let interface= interface.map(|i| { format!("|interface={}", i) }).unwrap_or(String::new());
            let arguments = Arguments {
                port: port.to_owned(),
                server: server.to_owned(),
                sforward: String::from(format!("endpoint=0.0.0.0:{}{}", port, interface)),
                cforward: String::from(format!("endpoint={}:{}{}", server, port, interface)),
                cbackward: String::from(format!("endpoint=0.0.0.0:{}{}", port, interface)),
                endpoint: matches.opt_str("endpoint").unwrap_or(String::from(format!("0.0.0.0:0"))),
            };
            info!("{:?}", arguments);
             if is_server {
                Some((Mode::Server, arguments))
            } else {
                Some((Mode::Client, arguments))
            }
        }
        Err(_) => { 
            print_usage(&program, opts);
            return None;
         }
    }
}
