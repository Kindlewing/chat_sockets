use std::io::Write;
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

fn handle_client_connection(mut stream: TcpStream) {
    let client_ip: IpAddr = match stream.peer_addr() {
        Ok(addr) => addr.ip(),
        Err(_) => panic!("There was an error"),
    };
    println!("Recieved a connection from {}", client_ip);
    let buffer = "Hello from server!".as_bytes();
    stream.write_all(buffer).unwrap();
}
