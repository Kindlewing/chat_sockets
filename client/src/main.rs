use std::io::{self, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;

const LOCAL: &str = "127.0.0.1:5555";

fn receive_message(server: &mut TcpStream) {
    let mut buff: Vec<u8> = vec![0; 1024];

    match server.read(&mut buff) {
        Ok(bytes_read) if bytes_read == 0 => {
            println!("Server disconnected");
            std::process::exit(0);
        }
        Ok(_) => {
            let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
            let msg_string: String = match String::from_utf8(msg) {
                Ok(str) => str,
                Err(err) => {
                    eprint!("String is not valid utf-8");
                    err.to_string()
                }
            };
            println!("{msg_string}");
        }
        Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
        Err(_) => {
            println!("Connection with server was severed");
            std::process::exit(1);
        }
    }
}

fn send_message(server: &mut TcpStream, rx: &mpsc::Receiver<String>) {
    match rx.try_recv() {
        Ok(msg) => {
            let mut buff = msg.clone().into_bytes();
            buff.resize(1024, 0);
            server.write_all(&buff).expect("writing to socket failed");
            server.flush().expect("Writing failed");
            println!("message sent {}", msg);
        }
        Err(TryRecvError::Empty) => (),
        Err(TryRecvError::Disconnected) => std::process::exit(1),
    }
}

fn main_loop(tx: mpsc::Sender<String>) {
    loop {
        let mut buff = String::new();
        io::stdin()
            .read_line(&mut buff)
            .expect("reading from stdin failed");
        let msg = buff.trim().to_string() + "\n";
        if msg == ":quit" || tx.send(msg).is_err() {
            break;
        }
    }
}

fn main() {
    let mut server = TcpStream::connect(LOCAL).expect("Stream failed to connect");
    server
        .set_nonblocking(true)
        .expect("failed to initiate non-blocking");

    let (tx, rx) = mpsc::channel::<String>();

    thread::spawn(move || loop {
        receive_message(&mut server);
        send_message(&mut server, &rx)
    });
    main_loop(tx);
    println!("bye bye!");
}
