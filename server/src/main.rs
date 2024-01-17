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

fn send_message_to_client(
    arc: &Arc<RwLock<Vec<String>>>,
    pos: &mut usize,
    stream: &mut BufStream<TcpStream>,
) -> std::io::Result<()> {
    let lines = arc.read().unwrap();
    for i in *pos..lines.len() {
        println!("Writing line: {}", lines[i]);
        stream.write_fmt(format_args!("{}", lines[i]))?;
    }
    *pos = lines.len();
    Ok(())
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
    write_from_server(stream, &format!("Hello, {}!\n\n", name))?;
    stream.flush()?;

    let mut pos = 0;
    loop {
        stream.write(b"> ".)?;
        stream.flush()?;
        let bytes_read = stream.read_line(&mut String::new())?;

        if bytes_read == 0 {
            println!("Client disconnected: {}", name);
            break Ok(());
        }
        send_message_to_client(&arc, &mut pos, stream)?;

        let mut reads = String::new();
        stream.read_line(&mut reads).unwrap(); // TODO: non-blocking read
        if reads.trim().len() != 0 {
            chan.send(format!("[{}] said: {}", name, reads)).unwrap();
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
