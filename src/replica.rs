use crate::cli;
use crate::parser::RespMessage;
use crate::tcp;
use std::io::Read;
use std::net::TcpStream;
use std::thread;

fn do_handshake(mut stream: &TcpStream, listening_port: &u16) {
    tcp::send_message(stream, RespMessage::new(String::from("PING")).build_reply()).unwrap();

    let mut buf = [0; 512];
    let handshake_response = tcp::read_message(stream, &mut buf);
    if handshake_response.trim() != "+PONG" {
        tcp::send_message(stream, String::from("-Wrong ping response")).unwrap()
    }

    tcp::send_message(
        stream,
        RespMessage::new(format!("REPLCONF listening-port {}", listening_port)).build_reply(),
    )
    .unwrap();

    let handshake_response = tcp::read_message(stream, &mut buf);
    println!("handshake: Recieved: {handshake_response}");

    tcp::send_message(
        stream,
        RespMessage::new(String::from("REPLCONF capa psycn2")).build_reply(),
    )
    .unwrap();
    let handshake_response = tcp::read_message(stream, &mut buf);
    println!("handshake: Received {handshake_response}");

    tcp::send_message(
        stream,
        RespMessage::new(String::from("PSYNC ? -1")).build_reply(),
    )
    .unwrap();

    let handshake_response = tcp::read_message(stream, &mut buf);
    println!("handshake: Received {handshake_response}");
    // let mut buf = Vec::new();
    // loop {
    //     match stream.read(&mut buf) {
    //         Ok(0) => {} // Connection closed
    //         Ok(_) => {
    //             let message = String::from_utf8_lossy(&buf);
    //             println!("Received: {message}");
    //             buf.clear(); // Clear the buffer for the next read
    //         }
    //         Err(e) => {
    //             eprintln!("Error reading from stream: {e}");
    //             break;
    //         }
    //     }
    // }
}

pub fn main_of_replica() {
    let args = cli::parse_cli();
    match args.replicaof {
        Some(replicaof) => {
            let stream =
                TcpStream::connect(format!("{}:{}", replicaof.host, replicaof.port)).unwrap();

            thread::spawn(move || {
                do_handshake(&stream, &args.port);
            });
        }
        _ => {}
    }
}
