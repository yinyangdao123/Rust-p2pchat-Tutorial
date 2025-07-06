use std::net::{TcpListener, TcpStream};
use std::io::{stdin, Read, Write};
use std::sync::{Arc, Mutex};
use std::thread::{self, sleep};
use std::time::Duration;




fn handle_client(option_stream:Arc<Mutex<Option<TcpStream>>>) {
    let mut buffer = [0; 512];
    loop {
        sleep(Duration::from_millis(500));
        let stream_lock = option_stream.lock().unwrap();

        match stream_lock.as_ref().unwrap().read(&mut buffer) {
            Ok(bytes_read) => {
                println!("{} reply: {}", stream_lock.as_ref().unwrap().peer_addr().unwrap(), str::from_utf8(&buffer[..bytes_read]).unwrap().trim());
                if bytes_read == 0 { break; }
            },
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                //什么也不做，让循环继续尝试
            }
            Err(e) => panic!("Encountered IO error: {e}"),
        }
    }
}


fn server(option_stream:Arc<Mutex<Option<TcpStream>>>) -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8081")?;
    println!("We are listening on: {}.", listener.local_addr().unwrap());

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        stream.set_nonblocking(true).unwrap();

        println!("We got incoming connection from {}.", stream.peer_addr().unwrap());
        {let mut stream_lock = option_stream.lock().unwrap();
             *stream_lock = Some(stream);}
        let option_stream_clone = Arc::clone(&option_stream);

        thread::spawn(move || {
            handle_client(option_stream_clone);
        });
    }
    Ok(())
}

fn connect()-> std::io::Result<TcpStream>{
    let stream = TcpStream::connect("127.0.0.1:8080")?;
    println!("We successfully connected to {:?}.", stream.peer_addr().unwrap());
    let mut stream_clone = stream.try_clone()?;

    thread::spawn(move || {
        let mut buffer = [0; 512];
        loop {
            let bytes_read = stream_clone.read(&mut buffer).unwrap();
            println!("{} reply: {}", stream_clone.peer_addr().unwrap(), String::from_utf8_lossy(&buffer[..bytes_read]).trim());
            if bytes_read == 0 { break; }
        }
    });

    Ok(stream)
}

fn main() -> std::io::Result<()> {
    let option_stream:Arc<Mutex<Option<TcpStream>>> = Arc::new(Mutex::new(None));
    let option_stream_clone = Arc::clone(&option_stream);

    thread::spawn(move ||{
        server(option_stream_clone).unwrap();
    });

    loop{
        let mut input = String::new();
        stdin().read_line(&mut input)?;

        let mut stream_lock = option_stream.lock().unwrap();

        match input.trim() {
            "connect" => {
                if let None = *stream_lock {
                    *stream_lock = Some(connect().unwrap());
                }
            }
            _=>{
                if let Some(ref mut stream) = *stream_lock {
                    stream.write_all(input.as_bytes())?;
                }
            }
        }
    }
}
