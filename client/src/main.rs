use std::net::TcpStream;

fn main() {
    let stream = TcpStream::connect("127.0.0.1:5555").expect("Failed to bind to address");
    println!(
        "Successfully connected to {}",
        stream.peer_addr().expect("Couldn't connect")
    );
}
