// Uncomment this block to pass the first stage
use std::{
    io::{Read, Write},
    net::TcpListener,
};

fn handle_conn(stream: &mut std::net::TcpStream) {
    let mut buf = [0; 512];
    loop {
        match stream.read(&mut buf) {
            Ok(0) => break,
            Ok(_) => {
                stream.write(b"+PONG\r\n").unwrap();
            }
            Err(e) => {
                println!("REDIS: error: {}", e);
                break;
            }
        }
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
