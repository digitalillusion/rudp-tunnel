use std::env;

use getopts::Options;
use log::info;

use rudp_tunnel::{Arguments, Mode, run};
use platform_dirs::AppDirs;

fn main() -> std::io::Result<()> {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info")
    );

    if let Some((mode, args)) = parse_args() {
        run(mode, args)
    }
    Ok(())
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
    opts.optopt("c", "control", "The control port used for client NAT traversal. Defaults to 32104", "CONTROL");
    opts.optopt("e", "endpoint", "Socket address where packets are sent/received, endpoint of the tunnel.", "ENDPOINT");
    opts.optopt("s", "server", "Public ip address of the server, implicitly defining this node as a client. Defaults to 0.0.0.0", "SERVER");
    opts.optopt("u", "public", "Public ip address of this node, starting as server. Ignored if SERVER is specified. Defaults to 0.0.0.0", "PUBLIC");
    opts.optopt("i", "interface", "Routing interface.", "INTERFACE");
    opts.optopt("m", "mtu", "Packets Maximum Transmission Unit. Defaults to 128 (bytes)", "MTU");
    opts.optopt("x", "maxclients", "Maximum number of simultaneously connected clients. Defaults to 10", "MAXCLIENTS");
    opts.optflag("l", "listen", "Defines whether to listen on the endpoint socket address instead of connecting");
    opts.optflag("d", "driverless", "Run without starting Aeron driver, assuming that it has been started externally.");
    opts.optflag("n", "nosharedmem", "Avoid using shared memory (/dev/shm) under Linux. Has no effect on other platforms.");

    match opts.parse(&args[1..]) {
        Ok(matches)  => {
            let is_server = !matches.opt_present("server");
            let port = matches.opt_str("port").map(|b| { b.parse::<usize>().unwrap() }).unwrap_or(40123);
            let control = matches.opt_str("control").map(|b| { b.parse::<usize>().unwrap() }).unwrap_or(32104);
            let server = matches.opt_str("server").unwrap_or(String::from("0.0.0.0"));
            let public = matches.opt_str("public").unwrap_or(String::from("0.0.0.0"));
            let interface = matches.opt_str("interface");
            let interface= interface.map(|i| { format!("|interface={}", i) }).unwrap_or(String::new());
            let mtu = matches.opt_str("mtu").unwrap_or(String::from("128")).parse().expect("Cannot parse mtu");
            let max_clients = matches.opt_str("maxclients").unwrap_or(String::from("10")).parse().expect("Cannot parse max clients");
            let arguments = Arguments {
                port: port.to_owned(),
                control: control.to_owned(),
                server: server.to_owned(),
                public: public.to_owned(),
                interface: interface.to_owned(),
                sforward: String::from(format!("endpoint=0.0.0.0:{}{}", port, interface)),
                sbackward: String::from(format!("endpoint={}:{}{}|control={}:{}|control-mode=dynamic", public, port, interface, public, control)),
                cforward: String::from(format!("endpoint={}:{}{}", server, port, interface)),
                cbackward: String::from(format!("endpoint=0.0.0.0:0{}|control={}:{}|control-mode=dynamic", interface, server, control)),
                listen: matches.opt_present("listen") || !matches.opt_present("endpoint"),
                endpoint: matches.opt_str("endpoint").unwrap_or(String::from(format!("0.0.0.0:0"))),
                driverless: matches.opt_present("driverless"),
                mtu,
                max_clients,
                dir_prefix: get_dir_prefix(matches.opt_present("nosharedmem")),
            };
            info!("{:?}", arguments);
            if matches.opt_present("help") {
                print_usage(&program, opts);
                None
            } else if is_server {
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

fn get_dir_prefix(no_shared_mem: bool) -> String {
    if cfg!(target_os = "windows") {
        let app_dirs = AppDirs::new(None, false).unwrap();
        String::from(format!("{}\\Temp\\aeron-{}\\", app_dirs.data_dir.to_str().unwrap() , whoami::username()))
    } else  {
        let dir = if no_shared_mem { "/tmp" } else { "/dev/shm" };
        String::from(format!("{}/aeron-{}", dir, whoami::username()))
    }
}
