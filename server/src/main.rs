use std::io::{Read, Write};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream},
    sync::{
        mpsc,
        mpsc::{Receiver, Sender},
    },
};

fn main() {
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 5555);
    let listener = TcpListener::bind(socket).expect("Failed to bind to address");
    println!("Server listening on {}:{}", socket.ip(), socket.port());
    let (sender, receiver): (Sender<String>, Receiver<String>) = mpsc::channel();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                std::thread::spawn(|| handle_client_connection(stream));
            }
            Err(e) => {
                eprintln!("Failed to establish connection: {}", e);
            }
        }
    }
}

fn handle_client_connection(mut stream: TcpStream) -> std::io::Result<()> {
    println!("Incoming connection from client {}", stream.peer_addr()?);
    let mut buffer: [u8; 1024] = [0; 1024];
    loop {
        let bytes_read: usize = stream.read(&mut buffer)?;
        if bytes_read == 0 {
            return Ok(());
        }
        let msg = match std::str::from_utf8(&buffer[..bytes_read]) {
            Ok(s) => s,
            Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
        };
        println!("{:?}", msg);
        let _ = stream.write(&buffer[..bytes_read]);
    }
}
