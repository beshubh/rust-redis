use std::io::Error;
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;

pub fn send_message(mut stream: &TcpStream, message: String) -> Result<(), Error> {
    stream.write(message.as_bytes())?;
    stream.flush()?;
    Ok(())
}

pub fn read_message(mut stream: &TcpStream) -> String {
    let mut buffer = [0; 1024];
    let size = stream
        .read(&mut buffer)
        .map_err(|e| {
            eprintln!("Error reading tcp stream: {}", e);
        })
        .unwrap();
    let res = std::str::from_utf8(&mut buffer[..size])
        .map_err(|e| {
            eprintln!("Error converting to string: {}", e);
        })
        .unwrap()
        .to_string()
        .to_owned();
    return res;
}
