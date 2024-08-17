use std::vec::Vec;

const BULK_STRING_BYTE: u8 = b'$';
const ARRAY_BYTE: u8 = b'*';

pub fn process(cmd: &[u8]) -> String {
    // *2\r\n$4\r\nECHO\r\n$3\r\nhey\r\n
    let mut args_to_read = 0;
    let mut cmd = cmd;
    if cmd[0] == ARRAY_BYTE {
        args_to_read = cmd[1] - b'0';
        cmd = &cmd[4..];
    }
    match cmd[0] {
        BULK_STRING_BYTE => process_bulk_string(cmd, args_to_read as usize),
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

fn process_bulk_string(cmd: &[u8], args_to_read: usize) -> String {
    let args = read_bulk_args(cmd, args_to_read);
    match args[0].to_lowercase().as_str() {
        "echo" => format!("+{}\r\n", args[1]),
        "ping" => "+PONG\r\n".to_string(),
        _ => "-ERR unknown command \r\n".to_string(),
    }
}
