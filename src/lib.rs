#[macro_use]
extern crate lazy_static;

use std::env::temp_dir;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use log::info;

use crate::client::Client;
use crate::server::Server;

mod aeron;
mod client;
mod server;

lazy_static! {
    static ref RUNNING: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
}

pub enum Mode {
    Client,
    Server,
}

#[derive(Clone, Debug)]
pub struct Arguments {
    pub port: i32,
    pub server: String,
    pub sforward: String,
    pub sbackward: String,
    pub cforward: String,
    pub cbackward: String,
    pub endpoint: String,
    pub driverless: bool,
}

pub fn run(mode: Mode, args: Arguments) {
    let running = RUNNING.clone();
    if args.driverless {
        info!("Skipping driver launch...");
        start_instance(running, mode, args);
    } else {
        info!("Launching Aeron driver...");
        let driver_path = extract_driver();

        let mut child = if cfg!(target_os = "windows") {
            let mut command = String::from("%JAVA_HOME%\\bin\\java -cp ");
            command.push_str(driver_path.as_str());
            command.push_str("%JVM_OPTS% io.aeron.driver.MediaDriver %*");
            Command::new("cmd")
                .args(&["/C", command.as_str()])
                .spawn()
                .expect("Error spawning Aeron driver process")
        } else {
            let mut command = String::from("${JAVA_HOME}/bin/java -cp ");
            command.push_str(driver_path.as_str());
            command.push_str("${JVM_OPTS} io.aeron.driver.MediaDriver \"$@\"");
            Command::new("sh")
                .arg("-c")
                .arg(command.as_str())
                .spawn()
                .expect("Error spawning Aeron driver process")
        };

        let transitory_duration = Duration::from_millis(1000);
        std::thread::sleep(transitory_duration);

        start_instance(RUNNING.clone(), mode, args);

        ctrlc::set_handler(move || {
            running.store(false, Ordering::SeqCst);
            child.kill().unwrap();
        }).unwrap();

    }
}

fn start_instance(running: Arc<AtomicBool>, mode: Mode, args: Arguments) {
    match mode {
        Mode::Client => Client::instance(args).start(running),
        Mode::Server => Server::instance(args).start(running),
    }
}

fn extract_driver() -> String {
    let bytes = include_bytes!("aeron-all-1.32.0-SNAPSHOT.jar");
    let mut driver_path = temp_dir();
    driver_path.push("aeron-driver.jar");
    let mut file = File::create(driver_path.to_owned()).expect("Error extracting Aeron driver jar");
    file.write_all(bytes).unwrap();
    String::from(driver_path.to_str().unwrap())
}