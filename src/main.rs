// Uncomment this block to pass the first stage
mod parser;

use parser::Data;
use std::sync::{Arc, Mutex};
use std::{
    collections::HashMap,
    io::{Read, Write},
    net::TcpListener,
};
use ulid::Ulid;

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

struct ServerOptions {
    port: u16,
    role: String,
    replica_of: Option<String>,
    master_replid: String,
    master_repl_offset: i64,
}

fn extract_server_options(command_args: Vec<String>) -> ServerOptions {
    let mut port = 6379;
    let mut role = "master";
    let mut replica_of: Option<String> = None;
    for i in 0..command_args.len() {
        match command_args[i].as_str() {
            "--replicaof" => {
                role = "slave";
                replica_of = Some(command_args[i + 1].clone());
            }
            "--port" => {
                port = command_args[i + 1].parse().unwrap();
            }
            _ => {}
        }
    }
    ServerOptions {
        port,
        role: role.to_string(),
        replica_of,
        master_repl_offset: 0,
        master_replid: Ulid::new().to_string(),
    }
}

fn init_server(
    data_store: &Arc<Mutex<HashMap<String, parser::Data>>>,
    server_options: ServerOptions,
) -> TcpListener {
    data_store.lock().unwrap().insert(
        "__role".to_string(),
        parser::Data {
            value: server_options.role.to_string(),
            exp: None,
        },
    );
    data_store.lock().unwrap().insert(
        String::from("__master_replid"),
        parser::Data {
            value: server_options.master_replid.clone(),
            exp: None,
        },
    );

    data_store.lock().unwrap().insert(
        String::from("__master_repl_offset"),
        Data {
            value: server_options.master_repl_offset.to_string(),
            exp: None,
        },
    );

    if let Some(replica_of) = server_options.replica_of.clone() {
        println!("I am a slave of {}", replica_of);
        data_store.lock().unwrap().insert(
            "__replicaof".to_string(),
            parser::Data {
                value: replica_of.clone(),
                exp: None,
            },
        );
    } else {
        println!("I am a master");
    }

    let addr = format!("127.0.0.1:{}", server_options.port);
    let listener = TcpListener::bind(addr).unwrap();
    listener
}

fn main() {
    let data_store = Arc::new(Mutex::new(HashMap::new()));

    let command_args: Vec<String> = std::env::args().collect();
    let server_options = extract_server_options(command_args);
    let listener = init_server(&data_store, server_options);

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
