use std::net::TcpStream;
use std::io::{stdin, Read, Write};
use std::thread;

fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:8080")?;
    let mut stream_clone = stream.try_clone()?;

    thread::spawn(move || {
        let mut buffer = [0; 512];
        loop {
            let bytes_read = stream_clone.read(&mut buffer).unwrap();
            if bytes_read == 0 { break; }
            print!("{}", String::from_utf8_lossy(&buffer[..bytes_read]));
        }
    });

    loop {
        let mut input = String::new();
        stdin().read_line(&mut input)?;
        stream.write_all(input.as_bytes())?;
    }
}
