use std::fs::{self, File};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::io::{BufRead, BufReader, ErrorKind, Read, Write};
use std::path::{self, Path};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::{env, thread};
use chapter10::app::App;
use chapter10::notice::{notice_and_prompt, Level};

fn register_name(app:Arc<Mutex<App>>, mut stream:TcpStream, local_name:String) {
    let local_bind_address = {app.lock().unwrap().get_local_bind_address()};
    let msg = format!("0{} {}", local_name, local_bind_address);
    stream.write_all(msg.as_bytes()).unwrap();
}

fn send_file(mut stream:TcpStream, path_str:String) {
    let path = Path::new(&path_str);
    if fs::exists(&path).is_ok() && Path::is_file(&path) {
        let file = File::open(path).unwrap();
        let file_length = file.metadata().unwrap().len();
        let mut buffer = [0; 1024]; 
        let mut file = BufReader::new(file);

        let msg = format!("3{} {}\n",path.file_name().unwrap().to_string_lossy(), file_length);
        stream.write_all(msg.as_bytes()).unwrap();
        
        loop {
            let count = file.read(&mut buffer).unwrap();
            if count == 0 {break;}
            stream.write_all(&buffer[..count]).unwrap();
        }
        stream.flush().unwrap();
        let notice = format!("[system]Finished sending {}", path.file_name().unwrap().to_string_lossy());
        notice_and_prompt(Some(&notice), Some(">"), Level::System);
        
    }
    else {
        let notice = format!("[system]{} does not exists", path_str);
        notice_and_prompt(Some(&notice), Some(">"), Level::System);
        
    }
}

fn generate_save_path(file_name:&str) -> String{
    let parent = env::current_dir().unwrap().to_string_lossy().to_string();
    let file_name = Path::new(file_name);
    let file_stem = file_name.file_stem().unwrap().to_string_lossy().to_string();
    let extension = file_name.extension().unwrap().to_string_lossy().to_string();

    let path = format!("{}/{}.{}", parent, file_stem, extension);
    let final_path = if !fs::exists(&path).unwrap() {
        path
    }
    else {
        let (head, mut i) = match file_stem.chars().rev().collect::<String>().split_once('_') {
            Some((tail, head)) => {
                let (tail, head) = (head.chars().rev().collect::<String>(), tail.chars().rev().collect::<String>());
                match tail.parse::<usize>() {
                    Ok(i)=>(head, i),
                    Err(_)=>(file_stem, 0)
                }
            },
            None=>(file_stem, 0)
        };
        loop {
            i=i+1;
            let new_path = format!("{}/{}_{}.{}", parent, head, i, extension);
            if !fs::exists(&new_path).unwrap() {
                break new_path;
            }
        }
    };
    path::absolute(final_path).unwrap().to_string_lossy().to_string()
}


fn handle_stream(mut stream:TcpStream, app:Arc<Mutex<App>>) {
    let remote_address = stream.peer_addr().unwrap();
    let mut stream_bufreader = BufReader::new(stream.try_clone().unwrap());
    
    loop {
        let mut buffer_head:[u8;1] = [0;1] ;

        match stream_bufreader.read_exact(&mut buffer_head) {
            Ok(())=>{
                match str::from_utf8(&buffer_head) {
                    Ok("0") => {
                        let mut buffer = [0;1024];
                        
                        match stream_bufreader.read(&mut buffer) {
                            Ok(count)=>{
                                if let Some((remote_name, remote_bind_address)) = str::from_utf8(&buffer[..count]).unwrap().trim().split_once(char::is_whitespace){
                                    if let Ok(remote_bind_address) = SocketAddr::from_str(remote_bind_address){
                                        let notice = format!("[system]{}(bind_port:{}) as name {}", remote_address, remote_bind_address.port(), remote_name);
                                        notice_and_prompt(Some(&notice), Some(">"), Level::System);
                                        
                                        {app.lock().unwrap().contact_list_insert_name_address(remote_name.to_string(), remote_address, remote_bind_address);}
                                    }
                                }
                            },
                            Err(_)=>{
                                let notice = format!("[system]{} disconnected", remote_address);
                                notice_and_prompt(Some(&notice), Some(">"), Level::System);
                                
                                {app.lock().unwrap().contact_list_remove_by_address(&remote_address);}
                                break;
                            }
                        }
                    },
                    Ok("1") => {
                        let mut buffer = [0;1024];
                        match stream_bufreader.read(&mut buffer) {
                            Ok(count)=>{
                                if let Some(remote_name) = {app.lock().unwrap().contact_list_get_name_by_address(&remote_address)} {
                                    let notice = format!("[{}]: {}", remote_name, str::from_utf8(&buffer[..count]).unwrap());
                                    notice_and_prompt(Some(&notice), Some(">"), Level::User);
                                }
                                else {
                                    let notice = format!("[{}]: {}", remote_address, str::from_utf8(&buffer[..count]).unwrap());
                                    notice_and_prompt(Some(&notice), Some(">"), Level::User);
                                }
                            }
                            Err(_)=>{
                                let notice = format!("[system]{} disconnected", remote_address);
                                notice_and_prompt(Some(&notice), Some(">"), Level::System);
                                
                                {app.lock().unwrap().contact_list_remove_by_address(&remote_address);}
                            }
                        }
                    },
                    Ok("2") => {
                        let contact_list = {app.lock().unwrap().get_contact_list()};
                        let msg = format!("1{}",contact_list);
                        stream.write_all(msg.as_bytes()).unwrap();
                    }
                    Ok("3") => {
                        let mut buffer_head_line = String::new();
                        let count = stream_bufreader.read_line(&mut buffer_head_line).unwrap();
                        if count ==0 {continue;}
                        
                        let (file_name, file_length) = buffer_head_line.split_once(char::is_whitespace).unwrap();
                        let (file_name, file_length) = (file_name.trim(), file_length.trim());
                        if let Some(remote_name) = {app.lock().unwrap().contact_list_get_name_by_address(&remote_address)} {
                            let notice = format!("[system]{} is sending file: {}, size: {}", remote_name, file_name, file_length);
                            notice_and_prompt(Some(&notice), Some(">"), Level::System);
                        }
                        else {
                            let notice = format!("[system]{} is sending file: {}, size: {}", remote_address, file_name, file_length);
                            notice_and_prompt(Some(&notice), Some(">"), Level::System);
                        }

                        let file_length:usize = file_length.parse().unwrap();
                        let mut buffer_file:Vec<u8> = vec![0; file_length];
                        stream_bufreader.read_exact(&mut buffer_file[..]).unwrap();

                        let path = generate_save_path(&file_name);
                        
                        let mut new_file = File::create(&path).unwrap();
                        new_file.write_all(&buffer_file).unwrap();
                        let notice = format!("[system]File is saved in {}", path);
                        notice_and_prompt(Some(&notice), Some(">"), Level::System);
                        
                    }
                    _=>{}//0 name 1 message 2 lookup 3 file
                }
            },
            Err(e)=>{
                {app.lock().unwrap().contact_list_remove_by_address(&remote_address);}

                match e.kind() {
                    ErrorKind::ConnectionAborted=>{
                        let notice = format!("[system]{} disconnected.", remote_address);
                        notice_and_prompt(Some(&notice), None, Level::System);
                    },
                    ErrorKind::ConnectionReset=>{
                        let notice = format!("[system]{} disconnected.", remote_address);
                        notice_and_prompt(Some(&notice), Some(">"), Level::System);
                    },
                    _=>{}
                }
                
                break; 
            }
        }

    }
}


fn bind(app:Arc<Mutex<App>>) {
    let local_bind_address = {app.lock().unwrap().get_local_bind_address()};
    let listener = TcpListener::bind(local_bind_address).unwrap();
    let notice = format!("[system]Listening on {}", listener.local_addr().unwrap());
    notice_and_prompt(Some(&notice), Some(">"), Level::System);
    

    for stream in listener.incoming() {
        match stream {
            Ok(stream)=>{
                //stream.set_nonblocking(true).unwrap();

                let notice = format!("[system]{} connected", stream.peer_addr().unwrap());
                notice_and_prompt(Some(&notice), Some(">"), Level::System);
                

                let local_name = {app.lock().unwrap().get_local_name()};
                register_name(app.clone(), stream.try_clone().unwrap(), local_name);
                
                let remote_address = stream.peer_addr().unwrap();
                let stream_clone = stream.try_clone().unwrap();
                
                {app.lock().unwrap().contact_list_insert_address_stream(remote_address, stream);}

                let app_clone = app.clone();
                thread::spawn(move || {
                    handle_stream(stream_clone, app_clone);
                });
            }
            Err(_)=>{
                break;
            }
        }
    }
}

fn connect(remote_address: SocketAddr, app:Arc<Mutex<App>>){
    let stream = TcpStream::connect(remote_address).unwrap();
    //stream.set_nonblocking(true).unwrap();

    let notice = format!("[system]{} connected", stream.peer_addr().unwrap());
    notice_and_prompt(Some(&notice), Some(">"), Level::System);
    

    let local_name = {app.lock().unwrap().get_local_name()};
    register_name(app.clone(), stream.try_clone().unwrap(), local_name);
    
    let remote_address = stream.peer_addr().unwrap();
    let stream_clone = stream.try_clone().unwrap();
    
    {app.lock().unwrap().contact_list_insert_address_stream(remote_address, stream);}
                
    let app_clone = app.clone();
    thread::spawn(move || {
        handle_stream(stream_clone, app_clone);
    });
}

fn main() -> std::io::Result<()> {
    let mut args = std::env::args();
    let (_, Some(local_bind_address), Some(name), None) = (args.next(), args.next(), args.next(), args.next())
    else {
        return Err(std::io::Error::other("Please run [peer ip:port name]"));
    };
    let local_bind_address = SocketAddr::from_str(local_bind_address.as_str().trim()).unwrap();
    let local_name = name.trim().to_string();

    let app = Arc::new(Mutex::new(App::new(local_bind_address, local_name)));
    let app_bind = app.clone();

    thread::spawn(move ||{
        bind(app_bind);
    });

    

    loop{
        let mut input = String::new();

        notice_and_prompt(None, Some(">"), Level::System);
        std::io::stdin().read_line(&mut input)?;
        

        if input.len() == 0 {
            continue;
        } else if input.len() > 1024 {
            let notice = format!("[system]Input message is too long!");
            notice_and_prompt(Some(&notice), Some(">"), Level::System);
            
            continue;
        }

        let (input_0, input_1) = input.trim().split_once(char::is_whitespace).unwrap_or((input.as_str().trim(), ""));
        let (input_0, input_1)= (input_0.trim(), input_1.trim());
        
        match input_0{
            "connect" =>{
                match SocketAddr::from_str(input_1.replace(char::is_whitespace, "").as_str()) {
                    Ok(remote_address)=>{
                        connect(remote_address, app.clone());
                    }
                    Err(_)=>{
                        let notice = format!("[system]Please run [connect ip:port]");
                        notice_and_prompt(Some(&notice), Some(">"), Level::System);
                        
                        continue;
                    }
                }
            }
            "quit" => {
                break;
            }
            "message" => {
                if let Some((remote_address_or_name, content)) = input_1.split_once(char::is_whitespace) {
                    let (remote_address_or_name, content) = (remote_address_or_name.trim(), content.trim());
                    let msg = format!("1{}", content);
                    if let Ok(remote_address) = SocketAddr::from_str(remote_address_or_name.replace(char::is_whitespace, "").as_str()) {
                        if let Some(mut stream) = {app.lock().unwrap().contact_list_get_stream_by_address(&remote_address)} {
                            stream.write_all(msg.as_bytes())?;
                        }
                    }
                    else if let Some(mut stream) = {app.lock().unwrap().contact_list_get_stream_by_name(&remote_address_or_name.to_string())} {
                        stream.write_all(msg.as_bytes())?;
                    }
                    else {
                        let notice = format!("[system]Please run [message ip:port|remote_name content]");
                        notice_and_prompt(Some(&notice), Some(">"), Level::System);
                        
                        continue;
                    }
                }
            }
            "list" => {
                {app.lock().unwrap().contact_list_display();}
            }
            "lookup" => {
                let remote_address_or_name = input_1.replace(char::is_whitespace, "");
                let msg = format!("2");
                if let Ok(remote_address) = SocketAddr::from_str(remote_address_or_name.replace(char::is_whitespace, "").as_str()) {
                    if let Some(mut stream) = {app.lock().unwrap().contact_list_get_stream_by_address(&remote_address)} {
                        stream.write_all(msg.as_bytes())?;
                    }
                }
                else if let Some(mut stream) = {app.lock().unwrap().contact_list_get_stream_by_name(&remote_address_or_name.to_string())} {
                    stream.write_all(msg.as_bytes())?;
                }
                else {
                    let notice = format!("[system]Please run [lookup ip:port|remote_name]");
                    notice_and_prompt(Some(&notice), Some(">"), Level::System);
                    
                    continue;
                }
            }
            "file" => {
                if let Some((remote_address_or_name, path)) = input_1.split_once(char::is_whitespace) {
                    let (remote_address_or_name, path) = (remote_address_or_name.trim(), path.trim());
                    
                    if let Ok(remote_address) = SocketAddr::from_str(remote_address_or_name.replace(char::is_whitespace, "").as_str()) {
                        if let Some(stream) = {app.lock().unwrap().contact_list_get_stream_by_address(&remote_address)} {
                            let path = path.to_string();
                            thread::spawn(move ||send_file(stream, path));
                        }
                    }
                    else if let Some(stream) = {app.lock().unwrap().contact_list_get_stream_by_name(&remote_address_or_name.to_string())} {
                        let path = path.to_string();
                        thread::spawn(move ||send_file(stream, path));
                    }
                    else {
                        let notice = format!("[system]Please run [file ip:port|remote_name path]");
                        notice_and_prompt(Some(&notice), Some(">"), Level::System);
                        
                        continue;
                    }
                }
            }
            _=>{}
        }
    }
    Ok(())
}
