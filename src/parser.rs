use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::vec::Vec;

const BULK_STRING_BYTE: u8 = b'$';
const ARRAY_BYTE: u8 = b'*';

pub fn process(cmd: &[u8], data_store: &Arc<Mutex<HashMap<String, String>>>) -> String {
    // *2\r\n$4\r\nECHO\r\n$3\r\nhey\r\n
    let mut args_to_read = 0;
    let mut cmd = cmd;
    if cmd[0] == ARRAY_BYTE {
        args_to_read = cmd[1] - b'0';
        cmd = &cmd[4..];
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
        let arg_args_to_read = (cmd[1] - b'0') as usize;
        let arg = std::str::from_utf8(&cmd[4..(4 + arg_args_to_read)]).unwrap();
        args.push(arg.to_string());
        args_to_read -= 1;
        cmd = &cmd[(4 + arg_args_to_read + 2)..];
    }
    args
}

fn process_bulk_string(
    data_store: &Arc<Mutex<HashMap<String, String>>>,
    cmd: &[u8],
    args_to_read: usize,
) -> String {
    let args = read_bulk_args(cmd, args_to_read);
    match args[0].to_lowercase().as_str() {
        "echo" => format!("+{}\r\n", args[1]),
        "ping" => "+PONG\r\n".to_string(),
        "set" => {
            let mut data_store = data_store.lock().unwrap();
            data_store.insert(args[1].clone(), args[2].clone());
            "+OK\r\n".to_string()
        }
        "get" => {
            let data_store = data_store.lock().unwrap();
            match data_store.get(&args[1]) {
                Some(value) => format!("+{}\r\n", value),
                None => "$-1\r\n".to_string(),
            }
        }
        _ => "-ERR unknown command \r\n".to_string(),
    }
}
