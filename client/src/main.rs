use std::{io::Write, net::TcpStream};

fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:5555").expect("Failed to bind to address");
    println!(
        "Successfully connected to {}",
        stream.peer_addr().expect("Couldn't connect")
    );
    let buffer = "Hello from client".as_bytes();
    let _ = stream.write(buffer);
}
