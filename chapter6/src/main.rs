use std::net::{SocketAddr, TcpListener, TcpStream};
use std::io::{stdin, Read, Write};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::thread::{self, sleep};
use std::time::Duration;
use chapter6::contact;




fn handle_stream(mut stream:TcpStream, contact_list:Arc<Mutex<Vec<(SocketAddr, TcpStream)>>>) {
    let mut buffer = [0; 512];
    loop {
        sleep(Duration::from_millis(500));
        
        match stream.read(&mut buffer) {
            Ok(bytes_read) => {
                if bytes_read == 0 { 
                    println!("{} disconnected", stream.peer_addr().unwrap());
                    contact::contact_list_remove(contact_list, (stream.peer_addr().unwrap(), stream));
                    break; 
                }
                println!("[{}]: {}", stream.peer_addr().unwrap(), str::from_utf8(&buffer[..bytes_read]).unwrap().trim());
            },
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                //什么也不做，让循环继续尝试
            }
            Err(_) => {
                println!("{} disconnected", stream.peer_addr().unwrap());
                contact::contact_list_remove(contact_list, (stream.peer_addr().unwrap(), stream));
                break;
            },
        }
    }
}


fn bind(local_address:SocketAddr, contact_list:Arc<Mutex<Vec<(SocketAddr, TcpStream)>>>) {
    let listener = TcpListener::bind(local_address).unwrap();
    println!("Listening on: {}", listener.local_addr().unwrap());

    for stream in listener.incoming() {
        match stream {
            Ok(stream)=>{
                stream.set_nonblocking(true).unwrap();

                println!("{} connected", stream.peer_addr().unwrap());
                let stream_clone = stream.try_clone().unwrap();
                let contact = (stream.peer_addr().unwrap(), stream_clone);
                
                contact::contact_list_push(contact_list.clone(), contact);

                let contact_list_clone = contact_list.clone();
                thread::spawn(move || {
                    handle_stream(stream, contact_list_clone);
                });
            }
            Err(_)=>{
                break;
            }
        }
    }
}

fn connect(remote_address: SocketAddr, contact_list:Arc<Mutex<Vec<(SocketAddr, TcpStream)>>>){
    let stream = TcpStream::connect(remote_address).unwrap();
    stream.set_nonblocking(true).unwrap();

    println!("{} connected", stream.peer_addr().unwrap());
    
    let stream_clone = stream.try_clone().unwrap();
    let contact = (stream.peer_addr().unwrap(), stream_clone);
    
    contact::contact_list_push(contact_list.clone(), contact);
                
    let contact_list_clone = contact_list.clone();
    thread::spawn(move || {
        handle_stream(stream, contact_list_clone);
    });
}

fn main() -> std::io::Result<()> {
    let mut args = std::env::args();
    let (_, Some(local_address), None) = (args.next(), args.next(), args.next())
    else {
        return Err(std::io::Error::other("Please run [peer ip:port]"));
    };
    let local_address = SocketAddr::from_str(local_address.as_str().trim()).unwrap();

    let contact_list:Arc<Mutex<Vec<(SocketAddr, TcpStream)>>> = Arc::new(Mutex::new(Vec::new()));
    let contact_list_bind = contact_list.clone();

    thread::spawn(move ||{
        bind(local_address, contact_list_bind);
    });

    loop{
        let mut input = String::new();
        stdin().read_line(&mut input)?;
        if input.len() == 0 {continue;}
        let (input_0, input_1) = input.trim().split_once(char::is_whitespace).unwrap_or((input.as_str().trim(), ""));
        let (input_0, input_1)= (input_0.trim(), input_1.trim());
        
        match input_0{
            "connect" =>{
                match SocketAddr::from_str(input_1.replace(char::is_whitespace, "").as_str()) {
                    Ok(remote_address)=>{
                        connect(remote_address, contact_list.clone());
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
            "message" => {
                if let Some((remote_address, content)) = input_1.split_once(char::is_whitespace) {
                    let (remote_address, content) = (remote_address.trim(), content.trim());
                    match SocketAddr::from_str(remote_address.replace(char::is_whitespace, "").as_str()) {
                        Ok(remote_address)=>{
                            if let Some(mut stream) = contact::contact_list_get_stream_by_socket_address(contact_list.clone(),remote_address){
                                stream.write_all(content.as_bytes())?;
                            }
                        }
                        Err(_)=>{
                            println!("Please run [message ip:port content]");
                            continue;
                        }
                    }
                }
            }
            "list"=>{
                contact::contact_list_display(contact_list.clone());
            }
            _=>{}
        }
    }
    Ok(())
}
