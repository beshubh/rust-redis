// Uncomment this block to pass the first stage
mod parser;

use std::sync::{Arc, Mutex};
use std::{
    collections::HashMap,
    io::{Read, Write},
    net::TcpListener,
};

fn handle_conn(
    data_store: Arc<Mutex<HashMap<String, parser::Data>>>,
    stream: &mut std::net::TcpStream,
) {
    let mut cmd = [0u8; 512];
    while let Ok(bytes_read) = stream.read(&mut cmd) {
        if bytes_read == 0 {
            break;
        }
        let value = parser::process(&cmd, &data_store);
        stream.write(value.as_bytes()).unwrap();
    }
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.

    let data_store = Arc::new(Mutex::new(HashMap::new()));
    let mut port = 6379;
    let command_args: Vec<String> = std::env::args().collect();
    if command_args.len() > 1 && command_args[1] == "--port" {
        port = command_args[2].parse().unwrap();
    }
    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(addr).unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                println!("REDIS: new connection");
                let data_store = data_store.clone();
                std::thread::spawn(move || handle_conn(data_store, &mut _stream));
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
