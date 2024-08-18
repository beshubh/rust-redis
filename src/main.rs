mod cli;
mod command;
mod parser;
mod replica;
mod store;
mod tcp;

use command::RedisCommand;
use std::{io::Read, net::TcpListener};
use tcp::send_message;

fn main() {
    let args = cli::parse_cli();
    let addr = format!("127.0.0.1:{}", args.port);
    replica::main_of_replica();
    let listener = TcpListener::bind(addr).unwrap();
    let store = store::Store::new();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let store = store.clone();
                let replicaof = args.replicaof.clone();
                std::thread::spawn(move || {
                    println!("REDIS: accpeted new connection");

                    loop {
                        let mut buf = [0; 1024];
                        let size = stream.read(&mut buf).unwrap_or(0);
                        if size == 0 {
                            break;
                        }

                        let resp = parser::parse_resp(&String::from_utf8_lossy(&buf))
                            .unwrap()
                            .1;
                        let command = command::parse_command(&resp).unwrap();

                        match command {
                            RedisCommand::Ping => {
                                if let Err(e) = send_message(&stream, String::from("+PONG\r\n")) {
                                    eprintln!("Error handling client: {}", e);
                                }
                            }
                            RedisCommand::Echo(message) => {
                                if let Err(e) = send_message(
                                    &stream,
                                    format!("${}\r\n{}\r\n", message.len(), message),
                                ) {
                                    eprintln!("Error handling client: {}", e);
                                }
                            }
                            RedisCommand::Get(key) => {
                                let mut message = String::from("$-1\r\n");
                                if let Some(value) = store.get(&key) {
                                    message = format!("${}\r\n{}\r\n", value.len(), value);
                                }
                                if let Err(e) = send_message(&stream, message) {
                                    eprint!("Error handling client: {}", e);
                                }
                            }
                            RedisCommand::Set(key, val, px) => {
                                store.set(key, val, px);
                                if let Err(e) = send_message(&stream, String::from("+OK\r\n")) {
                                    eprint!("Error handling client: {}", e);
                                }
                            }
                            RedisCommand::Info => {
                                let role = match replicaof {
                                    Some(_) => "slave",
                                    _ => "master",
                                };
                                let info = [
                                    format!("role:{}", role),
                                    "master_replid:8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb"
                                        .to_string(),
                                    "master_repl_offset:0".to_string(),
                                ]
                                .join("\r\n");
                                let message = format!("${}\r\n{}\r\n", info.len(), info);
                                if let Err(e) = send_message(&stream, message) {
                                    eprintln!("Error handling client: {}", e);
                                }
                            }
                            RedisCommand::ReplConf => {
                                if let Err(e) = send_message(&stream, String::from("+OK\r\n")) {
                                    eprintln!("Error handling client {}", e);
                                }
                            }
                            RedisCommand::Psycn => {
                                let message = String::from(
                                    "+FULLRESYNC 8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb 0\r\n",
                                );
                                if let Err(e) = send_message(&stream, message) {
                                    eprintln!("Error handling client: {}", e);
                                }
                            }
                        }
                    }
                });
            }
            Err(e) => {
                println!("Error listening to connection: {}", e);
            }
        }
    }
}
