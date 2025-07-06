use std::net::{TcpListener, TcpStream};
use std::io::{stdin, Read, Write};
use std::thread;


fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 512];
    loop {
        let bytes_read = stream.read(&mut buffer).unwrap();
        println!("We got '{}' from {}.", str::from_utf8(&buffer[..bytes_read]).unwrap().trim(), stream.peer_addr().unwrap());
        if bytes_read == 0 { break; }
        stream.write(&buffer[..bytes_read]).unwrap();
    }
}


fn server() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    println!("We are listening on: {}.", listener.local_addr().unwrap());

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        println!("We got incoming connection from {}.", stream.peer_addr().unwrap());

        thread::spawn(|| {
            handle_client(stream);
        });
    }
    Ok(())
}

fn connect()-> std::io::Result<TcpStream>{
    let stream = TcpStream::connect("127.0.0.1:8081")?;
    println!("We successfully connected to {:?}.", stream.peer_addr().unwrap());
    let mut stream_clone = stream.try_clone()?;

    thread::spawn(move || {
        let mut buffer = [0; 512];
        loop {
            let bytes_read = stream_clone.read(&mut buffer).unwrap();
            if bytes_read == 0 { break; }
            print!("{} reply: {}.", stream_clone.peer_addr().unwrap(), String::from_utf8_lossy(&buffer[..bytes_read]));
        }
    });

    Ok(stream)
}

fn main() -> std::io::Result<()> {
    //建立新线程，启动服务器
    thread::spawn(||{
        server().unwrap();
    });


    let mut option_stream:Option<TcpStream> = None;
    loop{
        let mut input = String::new();
        stdin().read_line(&mut input)?;
        
        match input.trim() {
            "connect" => {
                if let None = option_stream {
                    option_stream = Some(connect().unwrap());
                }
            }
            _=>{
                if let Some(ref mut stream) = option_stream {
                    stream.write_all(input.as_bytes())?;
                }
            }
        }
    }
}
