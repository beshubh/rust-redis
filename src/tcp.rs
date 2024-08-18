use std::io::Error;
use std::io::Write;
use std::net::TcpStream;

pub fn send_message(mut stream: &TcpStream, message: String) -> Result<(), Error> {
    stream.write(message.as_bytes())?;
    stream.flush()?;
    Ok(())
}
