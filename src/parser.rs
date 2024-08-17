use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use std::vec::Vec;

const BULK_STRING_BYTE: u8 = b'$';
const ARRAY_BYTE: u8 = b'*';

pub struct Data {
    value: String,
    exp: Option<u128>,
}

pub fn process(cmd: &[u8], data_store: &Arc<Mutex<HashMap<String, Data>>>) -> String {
    // *2\r\n$4\r\nECHO\r\n$3\r\nhey\r\n
    let mut args_to_read = 0;
    let mut cmd = cmd;
    if cmd[0] == ARRAY_BYTE {
        // iterate until we find crlf
        // $120\r\n
        let mut idx = 1;
        while &cmd[idx..idx + 2] != b"\r\n" {
            args_to_read = args_to_read * 10 + (cmd[idx] - b'0');
            idx += 1;
        }
        cmd = &cmd[idx + 2..];
    }
    match cmd[0] {
        BULK_STRING_BYTE => process_bulk_string(data_store, cmd, args_to_read as usize),
        _ => "-ERR unknown first command\r\n".to_string(),
    }
}

fn read_bulk_args(cmd: &[u8], args_to_read: usize) -> Vec<String> {
    let mut args = Vec::new();
    let mut args_to_read = args_to_read;
    let mut cmd = cmd;
    // $4\r\nECHO\r\n$3\r\nhey\r\n
    while args_to_read > 0 {
        let arg_len = (cmd[1] - b'0') as usize;
        let arg = std::str::from_utf8(&cmd[4..(4 + arg_len)]).unwrap();
        args.push(arg.to_string());
        args_to_read -= 1;
        cmd = &cmd[(4 + arg_len + 2)..];
    }
    args
}

fn process_bulk_string(
    data_store: &Arc<Mutex<HashMap<String, Data>>>,
    cmd: &[u8],
    args_to_read: usize,
) -> String {
    let args = read_bulk_args(cmd, args_to_read);
    match args[0].to_lowercase().as_str() {
        "echo" => format!("+{}\r\n", args[1]),
        "ping" => "+PONG\r\n".to_string(),
        "set" => handle_set(data_store, args),
        "get" => handle_get(data_store, args),
        _ => "-ERR unknown command \r\n".to_string(),
    }
}

fn handle_set(data_store: &Arc<Mutex<HashMap<String, Data>>>, args: Vec<String>) -> String {
    if args.len() < 3 {
        return "-ERR wrong number of arguments for 'set' command\r\n".to_string();
    }
    println!("I don't know what's wrong here: {:?}", args);
    let mut exp = None;
    if args.len() >= 4 {
        match args[3].to_lowercase().as_str() {
            "px" => {
                if args.len() < 5 {
                    return "-ERR wrong number of arguments for 'set' with 'px' command\r\n"
                        .to_string();
                }
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis();
                let millis: u128 = args[4].parse().expect("expiration is not an integer");
                exp = Some(now + millis);
            }
            _ => {
                return "-ERR unknown command \r\n".to_string();
            }
        }
    }

    let mut data_store: std::sync::MutexGuard<HashMap<String, Data>> = data_store.lock().unwrap();
    data_store.insert(
        args[1].clone(),
        Data {
            value: args[2].clone(),
            exp,
        },
    );
    "+OK\r\n".to_string()
}

fn handle_get(data_store: &Arc<Mutex<HashMap<String, Data>>>, args: Vec<String>) -> String {
    if args.len() < 2 {
        return "-ERR wrong number of arguments for 'get' command\r\n".to_string();
    }
    let mut data_store = data_store.lock().unwrap();
    match data_store.get(&args[1]) {
        Some(data) => {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();

            if Option::is_some(&data.exp) && data.exp < Some(now) {
                data_store.remove(&args[1]);
                return "$-1\r\n".to_string();
            } else {
                return format!("+{}\r\n", data.value);
            }
        }
        None => "$-1\r\n".to_string(),
    }
}
