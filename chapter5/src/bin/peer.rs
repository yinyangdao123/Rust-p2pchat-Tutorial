use std::net::{SocketAddr, TcpListener, TcpStream};
use std::io::{stdin, Read, Write};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::thread::{self, sleep};
use std::time::Duration;




fn handle_stream(option_stream:Arc<Mutex<Option<TcpStream>>>) {
    let mut buffer = [0; 512];
    loop {
        sleep(Duration::from_millis(500));
        let mut stream_lock = option_stream.lock().unwrap();

        match stream_lock.as_ref().unwrap().read(&mut buffer) {
            Ok(bytes_read) => {
                if bytes_read == 0 { 
                    println!("{} disconnected", stream_lock.as_ref().unwrap().peer_addr().unwrap());
                    *stream_lock=None;
                    break; 
                }
                println!("[{}]: {}", stream_lock.as_ref().unwrap().peer_addr().unwrap(), str::from_utf8(&buffer[..bytes_read]).unwrap().trim());
            },
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                //什么也不做，让循环继续尝试
            }
            Err(_) => {
                println!("{} disconnected", stream_lock.as_ref().unwrap().peer_addr().unwrap());
                *stream_lock=None;
                break;
            },
        }
    }
}


fn bind(local_address:SocketAddr, option_stream:Arc<Mutex<Option<TcpStream>>>) {
    let listener = TcpListener::bind(local_address).unwrap();
    println!("Listening on: {}", listener.local_addr().unwrap());

    for stream in listener.incoming() {
        match stream {
            Ok(stream)=>{
                stream.set_nonblocking(true).unwrap();

                println!("{} connected", stream.peer_addr().unwrap());
                {let mut stream_lock = option_stream.lock().unwrap();
                     *stream_lock = Some(stream);}
                let option_stream_clone = Arc::clone(&option_stream);
                
                thread::spawn(move || {
                    handle_stream(option_stream_clone);
                });
            }
            Err(_)=>{
                break;
            }
        }
    }
}

fn connect(remote_address: SocketAddr, option_stream:Arc<Mutex<Option<TcpStream>>>){
    let stream = TcpStream::connect(remote_address).unwrap();
    stream.set_nonblocking(true).unwrap();

    println!("{} connected", stream.peer_addr().unwrap());

    {let mut stream_lock = option_stream.lock().unwrap();
        *stream_lock = Some(stream);}
    let option_stream_clone = Arc::clone(&option_stream);
                
    thread::spawn(move || {
        handle_stream(option_stream_clone);
    });
}

fn main() -> std::io::Result<()> {
    let mut args = std::env::args();
    let (_, Some(local_address), None) = (args.next(), args.next(), args.next())
    else {
        return Err(std::io::Error::other("Please run [peer ip:port]"));
    };
    let local_address = SocketAddr::from_str(local_address.as_str().trim()).unwrap();

    let option_stream:Arc<Mutex<Option<TcpStream>>> = Arc::new(Mutex::new(None));
    let option_stream_bind = Arc::clone(&option_stream);

    thread::spawn(move ||{
        bind(local_address, option_stream_bind);
    });

    loop{
        let mut input = String::new();
        stdin().read_line(&mut input)?;
        if input.len() == 0 {continue;}
        let (input_0, input_1) = input.split_once(char::is_whitespace).unwrap_or((input.as_str().trim(), ""));
        let (input_0, input_1)= (input_0.trim(), input_1.trim());
        
        let option_stream_connect = Arc::clone(&option_stream);
        match input_0{
            "connect" =>{
                match SocketAddr::from_str(input_1.replace(char::is_whitespace, "").as_str()) {
                    Ok(remote_address)=>{
                        let is_none = { if let None = *option_stream.lock().unwrap() {true} else {false} };
                        if is_none {
                            connect(remote_address, option_stream_connect);
                        }
                    }
                    Err(_)=>{
                        println!("Please run [connect ip:port]");
                        continue;
                    }
                }
            }
            "quit" => {
                break;
            }
            _=>{
                if let Some(ref mut stream) = *option_stream.lock().unwrap() {
                    stream.write_all(input.as_bytes())?;
                }
            }
        }
    }
    Ok(())
}
