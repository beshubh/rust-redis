// Uncomment this block to pass the first stage
mod parser;

use std::{io::Read, io::Write, net::TcpListener};

fn handle_conn(stream: &mut std::net::TcpStream) {
    let mut cmd = [0u8; 512];
    while let Ok(bytes_read) = stream.read(&mut cmd) {
        if bytes_read == 0 {
            break;
        }
        let value = parser::process(&cmd);
        stream.write(value.as_bytes()).unwrap();
    }
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                println!("REDIS: new connection");
                std::thread::spawn(move || handle_conn(&mut _stream));
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
