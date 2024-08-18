use crate::cli;
use crate::parser::RespMessage;
use crate::tcp;
use std::io::Read;
use std::net::TcpStream;

fn do_handshake(mut stream: &TcpStream, listening_port: &u16) {
    let resp_msg = RespMessage::new(String::from("PING")).build_reply();
    tcp::send_message(stream, resp_msg).unwrap();

    let mut buf = [0; 512];
    if let Ok(read_bytes) = stream.read(&mut buf) {
        let response = std::str::from_utf8(&buf[..read_bytes]).unwrap();
        println!("handshake: Received {response}");

        if response.trim() != "+PONG" {
            tcp::send_message(stream, String::from("-Error wrong response for PING")).unwrap();
        }
        let replconf_port =
            RespMessage::new(format!("REPLCONF listening-port {}", listening_port)).build_reply();

        tcp::send_message(stream, replconf_port).unwrap();
        let replconf_capa_psycn2 =
            RespMessage::new(String::from("REPLCONF capa psycn2")).build_reply();
        stream.read(&mut buf).unwrap();
        tcp::send_message(stream, replconf_capa_psycn2).unwrap();

        let mut buf = [0; 512];
        if let Ok(read_bytes) = stream.read(&mut buf) {
            let response = std::str::from_utf8(&buf[..read_bytes]).unwrap();
            println!("handshake: Received: {response}");
            let psync_msg = RespMessage::new(String::from("PSYNC ? -1")).build_reply();
            println!("psync last message: {}", psync_msg);
            tcp::send_message(stream, psync_msg).unwrap();
        } else {
            eprintln!("handshake: Error reading replconf response")
        }
    } else {
        eprintln!("handshake: error reading from master while handshake")
    }
}

pub fn main_of_replica() {
    let args = cli::parse_cli();
    match args.replicaof {
        Some(replicaof) => {
            let stream =
                TcpStream::connect(format!("{}:{}", replicaof.host, replicaof.port)).unwrap();
            do_handshake(&stream, &args.port);
        }
        _ => {}
    }
}
