use bufstream::BufStream;
use std::io::ErrorKind;
use std::io::{BufRead, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::str::FromStr;
use std::sync::{
    mpsc,
    mpsc::{Receiver, Sender},
    Arc, RwLock,
};

fn write_from_server(
    stream: &mut BufStream<TcpStream>,
    buffer: &str,
) -> Result<usize, std::io::Error> {
    let server_msg = format!("[SERVER]  {}\n", buffer);
    stream.write(server_msg.as_bytes())
}

fn handle_connection(
    stream: &mut BufStream<TcpStream>,
    chan: Sender<String>,
    arc: Arc<RwLock<Vec<String>>>,
) -> std::io::Result<()> {
    write_from_server(stream, "Welcome to Simple Chat Server! ")?;
    write_from_server(stream, "Please input yourname: ")?;
    stream.flush()?;
    let mut name = String::new();
    if let Err(err) = stream.read_line(&mut name) {
        eprintln!("Error reading line: {}", err);
        return Err(err);
    }
    let name = name.trim_end();
    println!("Name: {}", name);
    stream
        .write_fmt(format_args!("Hello, {}!\n", name))
        .unwrap();
    stream.flush()?;

    let mut pos = 0;
    loop {
        let bytes_read = stream.read_line(&mut String::new())?;
        if bytes_read == 0 {
            println!("Client disconnected: {}", name);
            break Ok(());
        }
        {
            println!("Inside chat loop");
            let lines = arc.read().unwrap();
            println!("DEBUG arc.read() => {:?}", lines);
            for i in pos..lines.len() {
                stream.write_fmt(format_args!("{}", lines[i]))?;
            }
            pos = lines.len();
        }
        stream.write(b" > ").unwrap();
        stream.flush()?;

        let mut reads = String::new();
        stream.read_line(&mut reads).unwrap(); //TODO: non-blocking read
        if reads.trim().len() != 0 {
            println!("DEBUG: reads len =>>>>> {}", reads.len());
            chan.send(format!("[{}] said: {}", name, reads)).unwrap();
            println!("DEBUG: got '{}' from {}", reads.trim(), name);
        }
    }
}

fn main() {
    let addr: SocketAddr = SocketAddr::from_str("127.0.0.1:5555").unwrap();
    let listener = TcpListener::bind(addr).unwrap();

    let (send, recv): (Sender<String>, Receiver<String>) = mpsc::channel();
    let arc: Arc<RwLock<Vec<String>>> = Arc::new(RwLock::new(Vec::new()));

    let arc_w = arc.clone();

    println!("Spawning thread");
    std::thread::spawn(move || {
        println!("Inside thread 1!");
        loop {
            let msg = recv.recv().unwrap();
            println!("DEBUG: msg {}", msg);
            {
                let mut arc_w = arc_w.write().unwrap();
                arc_w.push(msg);
            } // write lock is released at the end of this scope
        }
    });

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!(
                    "connection from {} to {}",
                    stream.peer_addr().unwrap(),
                    stream.local_addr().unwrap()
                );
                let send = send.clone();
                let arc = arc.clone();
                std::thread::spawn(move || {
                    let mut stream = BufStream::new(stream);
                    let _ = handle_connection(&mut stream, send, arc);
                });
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                println!("a client disconnected");
                continue;
            }
            Err(e) => panic!("encountered IO error: {e}"),
        }
    }
}
