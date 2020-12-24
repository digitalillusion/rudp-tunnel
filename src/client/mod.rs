use crate::Arguments;
use std::net::UdpSocket;
use std::time::Duration;
use crate::aeron::Settings;

pub fn instance (args: Arguments) {
    let net_addr = format!("0.0.0.0:{}", args.port);
    let socket = UdpSocket::bind(net_addr.to_owned()).unwrap();
    socket.set_nonblocking(true).expect("Failed to enter non-blocking mode");

    let settings = Settings::new(args);
    let publisher = crate::aeron::publisher::Publisher::new(settings.clone()).unwrap();

    println!("Client listening on {} ", net_addr);
    loop {
        let mut recv_buff = [0; 256];
        if let Ok((n, addr)) = socket.recv_from(&mut recv_buff) {
            println!("{} bytes received from {:?}", n, addr);
            publisher.publish(recv_buff, n as i32);
        }

        std::thread::sleep(Duration::from_millis(1));
    }
}