use std::error::Error;
use std::fmt::write;
use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, TcpListener, TcpStream};
use std::sync::{
    mpsc,
    mpsc::{Receiver, Sender},
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:5555").expect("Failed to bind to address");
    println!("Server listening on 127.0.0.1:5555");
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
    let mut buffer: [u8; 512] = [0; 512];
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
        stream.write(&buffer[..bytes_read]);
    }
}
