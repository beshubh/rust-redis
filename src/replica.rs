use crate::cli;
use crate::parser::RespMessage;
use crate::tcp;
use std::io::Read;
use std::net::TcpStream;
use std::thread;

pub fn do_handshake(mut stream: &TcpStream, listening_port: &u16) {
    tcp::send_message(stream, RespMessage::new(String::from("PING")).build_reply()).unwrap();

    let handshake_response = tcp::read_message(stream);
    println!("handshake: Recieved ping reponse: {handshake_response}");
    if handshake_response.trim() != "+PONG" {
        tcp::send_message(stream, String::from("-Wrong ping response")).unwrap()
    }

    tcp::send_message(
        stream,
        RespMessage::new(format!("REPLCONF listening-port {}", listening_port)).build_reply(),
    )
    .unwrap();

    let handshake_response = tcp::read_message(stream);
    println!("handshake: Recieved replconf port response: {handshake_response}");

    tcp::send_message(
        stream,
        RespMessage::new(String::from("REPLCONF capa psycn2")).build_reply(),
    )
    .unwrap();
    let handshake_response = tcp::read_message(stream);
    println!("handshake: Received capa psycn2 reponse {handshake_response}");

    tcp::send_message(
        stream,
        RespMessage::new(String::from("PSYNC ? -1")).build_reply(),
    )
    .unwrap();

    // read and ignore empty rdb file
    let mut buf = [0; 1024];
    stream.read(&mut buf).unwrap();
    let mut buf = [0; 1024];
    stream.read(&mut buf).unwrap();
}

pub fn main_of_replica() {
    let args = cli::parse_cli();
    match args.replicaof {
        Some(replicaof) => {
            let mut stream =
                TcpStream::connect(format!("{}:{}", replicaof.host, replicaof.port)).unwrap();

            thread::spawn(move || {
                do_handshake(&stream, &args.port);
                loop {
                    let mut buf = Vec::new();

                    let size = stream.read(&mut buf).unwrap();
                    if size == 0 {
                        // println!("no bytes to read");
                        continue;
                    } else {
                        let data = &String::from_utf8_lossy(&buf);
                        println!("data received: {}", data);
                    }
                }
            });
        }
        _ => {}
    }
}
