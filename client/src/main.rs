use std::io::{self, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;

const LOCAL: &str = "127.0.0.1:5555";
const MSG_SIZE: usize = 1024;

fn connect_to_server() -> TcpStream {
    TcpStream::connect(LOCAL).expect("Stream failed to connect")
}

fn set_nonblocking(stream: &mut TcpStream) {
    stream
        .set_nonblocking(true)
        .expect("failed to initiate non-blocking");
}

fn receive_message(client: &mut TcpStream, rx: &mpsc::Receiver<String>) {
    let mut buff = vec![0; MSG_SIZE];
    match client.read(&mut buff) {
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
            println!("connection with server was severed");
            std::process::exit(1);
        }
    }

    match rx.try_recv() {
        Ok(msg) => {
            let mut buff = msg.clone().into_bytes();
            buff.resize(MSG_SIZE, 0);
            client.write_all(&buff).expect("writing to socket failed");
            client.flush().expect("Writing failed");
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
    let mut client = connect_to_server();
    set_nonblocking(&mut client);

    let (tx, rx) = mpsc::channel::<String>();

    thread::spawn(move || loop {
        receive_message(&mut client, &rx);
    });
    main_loop(tx);
    println!("bye bye!");
}
