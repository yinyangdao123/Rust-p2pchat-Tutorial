use std::{net::{SocketAddr, UdpSocket}};

fn main() -> std::io::Result<()> {    
    // 创建服务器套接字
    let server_addr: SocketAddr = "0.0.0.0:23456".parse().unwrap();
    let server = UdpSocket::bind(server_addr)?;
    
    // 服务器接收数据
    let mut buf = [0u8; 1024];
    while let Ok((size, src_addr)) = server.recv_from(&mut buf) {
        println!("Received {} bytes from {}: {}", size, src_addr, str::from_utf8(&buf[..size]).unwrap());
        if size==0 {
            break;
        }
    }
    Ok(())
}