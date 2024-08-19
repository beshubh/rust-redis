mod cli;
mod command;
mod parser;
mod replica;
mod store;
mod tcp;

use cli::ReplicaInfo;
use command::RedisCommand;
use hex;
use parser::RespMessage;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use store::Store;
use tcp::send_message;

fn handle_connection(
    mut stream: &TcpStream,
    store: &Store,
    slaves: &Arc<Mutex<Vec<TcpStream>>>,
    replicaof: Option<ReplicaInfo>,
    wal_buffer: &Arc<Mutex<Vec<String>>>,
) {
    loop {
        let mut buf = [0; 1024];
        let size = stream.read(&mut buf).unwrap_or(0);
        if size == 0 {
            println!("REDIS: closing connection");
            break;
        }

        let resp = parser::parse_resp(&String::from_utf8_lossy(&buf))
            .unwrap()
            .1;
        let command = command::parse_command(&resp);
        if command.is_none() {
            eprintln!("invalid command received");
            break;
        }
        let command = command.unwrap();

        match command {
            RedisCommand::Ping => {
                if let Err(e) = send_message(&stream, String::from("+PONG\r\n")) {
                    eprintln!("Error handling client: {}", e);
                }
            }
            RedisCommand::Echo(message) => {
                if let Err(e) =
                    send_message(&stream, format!("${}\r\n{}\r\n", message.len(), message))
                {
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
                store.set(key.clone(), val.clone(), px);
                if !replicaof.is_some() {
                    if let Err(e) = send_message(&stream, String::from("+OK\r\n")) {
                        eprint!("Error handling client: {}", e);
                    }
                }
                if replicaof.is_none() {
                    let mut wal_buffer = wal_buffer.lock().unwrap();
                    let mut replication_command = format!("SET {} {}", key.clone(), val.clone());

                    if px.is_some() {
                        replication_command = format!("{} PX {}", replication_command, px.unwrap())
                    }
                    wal_buffer.push(replication_command);
                }
            }
            RedisCommand::Info => {
                let role = match replicaof {
                    Some(_) => "slave",
                    _ => "master",
                };
                let info = [
                    format!("role:{}", role),
                    "master_replid:8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb".to_string(),
                    "master_repl_offset:0".to_string(),
                ]
                .join("\r\n");
                let message = format!("${}\r\n{}\r\n", info.len(), info);
                if let Err(e) = send_message(&stream, message) {
                    eprintln!("Error handling client: {}", e);
                }
            }
            RedisCommand::ReplConf(key, value) => {
                println!("REPLCONF: {} {}", key, value);
                if key == "listening-port" {
                    println!("REPLCONF: adding to slaves: {}", value);
                    let mut slaves = slaves.lock().unwrap();
                    slaves.push(stream.try_clone().unwrap());
                }
                if let Err(e) = send_message(&stream, String::from("+OK\r\n")) {
                    eprintln!("Error handling client {}", e);
                }
            }
            RedisCommand::Psync => {
                psync(&stream);
            }
        }
    }
}

fn psync(mut stream: &TcpStream) {
    let message = String::from("+FULLRESYNC 8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb 0\r\n");
    if let Err(e) = send_message(&stream, message) {
        eprintln!("Error handling client: {}", e);
    }
    let empty_rdb_hex_str = hex::decode("524544495330303131fa0972656469732d76657205372e322e30fa0a72656469732d62697473c040fa056374696d65c26d08bc65fa08757365642d6d656dc2b0c41000fa08616f662d62617365c000fff06e3bfec0ff5aa2").unwrap();
    stream
        .write(format!("${}\r\n", empty_rdb_hex_str.len()).as_bytes())
        .unwrap();
    stream.write(&empty_rdb_hex_str).unwrap();
    stream.flush().unwrap();
}

fn clear_wal(wal_buffer: &Arc<Mutex<Vec<String>>>) {
    let mut wal_buffer = wal_buffer.lock().unwrap();
    wal_buffer.clear()
}

fn wal_replication(wal_buffer: &Arc<Mutex<Vec<String>>>, slaves: &Arc<Mutex<Vec<TcpStream>>>) {
    let slaves = slaves.lock().unwrap();
    let wal_buffer = wal_buffer.lock().unwrap();
    // println!("{:?}", wal_buffer);
    for stream in slaves.iter() {
        for cmd in wal_buffer.iter() {
            let cmd = cmd.clone();
            let cmd = RespMessage::new(cmd).build_reply();
            send_message(stream, cmd)
                .map_err(|e| eprintln!("ErrorReplication: cannot send command to replica: {}", e))
                .unwrap();
        }
    }
}

fn main() {
    let args = cli::parse_cli();
    let addr = format!("127.0.0.1:{}", args.port);

    let listener = TcpListener::bind(addr).unwrap();
    let replicaof = args.replicaof.clone();
    let store = store::Store::new();
    let slaves: Arc<Mutex<Vec<TcpStream>>> = Arc::new(Mutex::new(Vec::new()));
    let wal_buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    if replicaof.is_some() {
        let host = replicaof.clone().unwrap().host;
        let port = replicaof.clone().unwrap().port;

        let stream = TcpStream::connect(format!("{}:{}", host, port));
        match stream {
            Ok(stream) => {
                let store = store.clone();
                let slaves = Arc::clone(&slaves);
                let replicaof = args.replicaof.clone();
                let wal_buffer = Arc::clone(&wal_buffer);

                replica::do_handshake(&stream, &args.port);
                std::thread::spawn(move || {
                    println!("replica: connected to master");
                    handle_connection(&stream, &store, &slaves, replicaof, &wal_buffer);
                });
            }
            Err(e) => {
                println!("Error connecting to master: {}", e);
            }
        }
    }
    {
        let _wal_buffer = Arc::clone(&wal_buffer);
        let _slaves = Arc::clone(&slaves);
        std::thread::spawn(move || loop {
            std::thread::sleep(std::time::Duration::from_millis(1000));
            wal_replication(&_wal_buffer, &_slaves);
        });

        let _wal_buffer = Arc::clone(&wal_buffer);
        std::thread::spawn(move || loop {
            std::thread::sleep(std::time::Duration::from_secs(200));
            clear_wal(&_wal_buffer);
        });
    }

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let store = store.clone();
                let slaves = Arc::clone(&slaves);
                let replicaof = args.replicaof.clone();
                let wal_buffer = Arc::clone(&wal_buffer);
                std::thread::spawn(move || {
                    println!("REDIS: accpeted new connection");
                    handle_connection(&stream, &store, &slaves, replicaof, &wal_buffer);
                });
            }
            Err(e) => {
                println!("Error listening to connection: {}", e);
            }
        }
    }
}
