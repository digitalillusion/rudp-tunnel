use crate::Arguments;

use aeron_rs::concurrent::atomic_buffer::AtomicBuffer;
use aeron_rs::utils::types::Index;
use aeron_rs::concurrent::logbuffer::header::Header;
use std::slice;
use crate::aeron::Settings;

use std::net::UdpSocket;

pub fn instance (args: Arguments) {
    let settings = Settings::new(args.clone());

    let socket:UdpSocket = UdpSocket::bind("0.0.0.0:0").unwrap();
    socket.connect(args.tunnel).unwrap();
    socket.set_nonblocking(true).unwrap();

    let on_new_fragment = |buffer: &AtomicBuffer, offset: Index, length: Index, header: &Header| {
        println!("{} bytes received on server from aeron", length);
        unsafe {
            let slice_msg = slice::from_raw_parts_mut(buffer.buffer().offset(offset as isize), length as usize);
            socket.send(slice_msg).unwrap();

            println!(
                "Message to stream {} from session {} ({}@{})",
                header.stream_id(),
                header.session_id(),
                length,
                offset,
            );
        }
    };

    let subscriber = crate::aeron::subscriber::Subscriber::new(settings).unwrap();
    println!("Server up");

    subscriber.listen(&on_new_fragment);

    println!("Server down");
}