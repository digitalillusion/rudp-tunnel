use std::cell::RefCell;
use std::net::UdpSocket;
use std::slice;
use std::time::Duration;

use aeron_rs::concurrent::atomic_buffer::AtomicBuffer;
use aeron_rs::concurrent::logbuffer::header::Header;
use aeron_rs::utils::types::Index;

use crate::aeron::Settings;
use crate::Arguments;

pub fn instance (args: Arguments) {
    let settings = Settings::new(args.to_owned());
    let net_addr = format!("0.0.0.0:{}", args.port);
    let origin_addr = RefCell::new(None);
    let socket_in = UdpSocket::bind(net_addr.to_owned()).expect("Error binding input socket");
    socket_in.set_nonblocking(true).expect("Failed to enter non-blocking mode for input socket");
    let socket_out = UdpSocket::bind("0.0.0.0:0").expect("Error binding output socket");
    socket_out.set_nonblocking(true).expect("Failed to enter non-blocking mode for output socket");

    let channel_forward = format!("aeron:udp?endpoint={}", args.forward);
    let channel_backward = format!("aeron:udp?endpoint={}", args.backward);

    let on_new_fragment = |buffer: &AtomicBuffer, offset: Index, length: Index, header: &Header| {
        let origin_addr = origin_addr.borrow().expect("Origin address is not specified but destination is sending packages.");
        dbg!("{} bytes received on stream {} from session {} toward {}", length, header.stream_id(), header.session_id(), origin_addr);
        unsafe {
            let slice_msg = slice::from_raw_parts_mut(buffer.buffer().offset(offset as isize), length as usize);
            socket_out.send_to(slice_msg, origin_addr).unwrap();
        }
    };

    let publisher = crate::aeron::publisher::Publisher::new(settings.clone(), channel_forward).expect("Error creating publisher");
    let publication = publisher.publish();
    let subscriber = crate::aeron::subscriber::Subscriber::new(settings.clone(), channel_backward).expect("Error creating subscriber");
    let subscription = subscriber.listen();
    println!("Client listening on {} ", net_addr);

    loop {
        let mut recv_buff = [0; 256];
        if let Ok((n, addr)) = socket_in.recv_from(&mut recv_buff) {
            dbg!("{} bytes received from {:?}", n, addr);
            origin_addr.borrow_mut().replace(addr);
            publisher.send(publication.to_owned(), recv_buff, n as i32);
        }

        subscriber.recv(subscription.to_owned(), &on_new_fragment);

        std::thread::sleep(Duration::from_millis(1));
    }
}