use std::io::{BufRead, BufReader};
use std::{io::Write, net::TcpStream};

fn main() {
    let mut stream = TcpStream::connect("server_ip:5555").expect("Failed to bind to address");
    loop {
        chat_loop(&mut stream);
    }
}

fn chat_loop(mut stream: &TcpStream) {
    /* only if first time connecting
     * Prompt user for username
     * add user to some sort of db
     */

    let mut msg: String = String::new();
    let mut server_buffer: Vec<u8> = Vec::new();
    std::io::stdin()
        .read_line(&mut msg)
        .expect("Unable to read input");
    stream
        .write(msg.as_bytes())
        .expect("Couldn't write to server");
    let mut reader = BufReader::new(&mut stream);
    reader
        .read_until(b'\n', &mut server_buffer)
        .expect("Couldn't read from server");
    print!(
        "{}",
        std::str::from_utf8(&server_buffer).expect("Could not write buffer as string")
    );
}
