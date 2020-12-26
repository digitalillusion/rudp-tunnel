use std::net::UdpSocket;
use std::slice;
use std::time::Duration;

use aeron_rs::concurrent::atomic_buffer::AtomicBuffer;
use aeron_rs::concurrent::logbuffer::header::Header;
use aeron_rs::utils::types::Index;

use crate::aeron::Settings;
use crate::Arguments;

pub fn instance (args: Arguments) {
    let settings = Settings::new(args.clone());
    let socket:UdpSocket = UdpSocket::bind("0.0.0.0:0").unwrap();
    socket.connect(args.tunnel.to_owned()).unwrap();
    socket.set_nonblocking(true).unwrap();

    let channel_forward = format!("aeron:udp?endpoint={}", args.forward);
    let channel_backward = format!("aeron:udp?endpoint={}", args.backward);

    let on_new_fragment = |buffer: &AtomicBuffer, offset: Index, length: Index, header: &Header| {
        dbg!("{} bytes received on stream {} from session {} toward {}", length, header.stream_id(), header.session_id(), args.tunnel.to_owned());
        unsafe {
            let slice_msg = slice::from_raw_parts_mut(buffer.buffer().offset(offset as isize), length as usize);
            socket.send(slice_msg).unwrap();
        }
    };

    let publisher = crate::aeron::publisher::Publisher::new(settings.clone(), channel_backward).expect("Error creating publisher");
    let publication = publisher.publish();
    let subscriber = crate::aeron::subscriber::Subscriber::new(settings.clone(), channel_forward).expect("Error creating subscriber");
    let subscription = subscriber.listen();
    println!("Server up and running, destination {}", args.tunnel);

    loop {
        let mut recv_buff = [0; 256];
        if let Ok((n, addr)) = socket.recv_from(&mut recv_buff) {
            dbg!("{} bytes received from {:?}", n, addr);
            publisher.send(publication.to_owned(), recv_buff, n as i32);
        }

        subscriber.recv(subscription.to_owned(), &on_new_fragment);

        std::thread::sleep(Duration::from_millis(1));
    }
}