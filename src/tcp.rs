use std::io::Error;
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;

pub fn send_message(mut stream: &TcpStream, message: String) -> Result<(), Error> {
    stream.write(message.as_bytes())?;
    stream.flush()?;
    Ok(())
}

pub fn read_message<'a>(mut stream: &'a TcpStream, buffer: &'a mut [u8]) -> &'a str {
    let size = stream
        .read(buffer)
        .map_err(|e| {
            eprintln!("Error reading tcp stream: {}", e);
        })
        .unwrap();
    std::str::from_utf8(&buffer[..size])
        .map_err(|e| {
            eprintln!("Error converting to string: {}", e);
        })
        .unwrap()
}
