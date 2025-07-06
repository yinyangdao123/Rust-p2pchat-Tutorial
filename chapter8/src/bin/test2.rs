use std::{net::{UdpSocket}, thread::sleep, time::Duration};

fn main() -> std::io::Result<()> {    

    sleep(Duration::from_secs(10));
    // 创建客户端套接字
    let client = UdpSocket::bind("0.0.0.0:0")?;
    // 客户端发送数据
    client.send_to(b"Hello I'm 34567!", "127.0.0.1:23456")?;

    
    Ok(())
}