use std::{net::{SocketAddr, TcpStream}, sync::{Arc, Mutex}};

pub fn contact_list_remove(contact_list:Arc<Mutex<Vec<(SocketAddr, TcpStream)>>>, contact:(SocketAddr, TcpStream)) {
    let mut lock = contact_list.lock().unwrap();
    lock.sort_by_key(|k|k.0);
    let i = lock.binary_search_by_key(&contact.0, |k|k.0).unwrap();
    lock.remove(i);
}

pub fn contact_list_push(contact_list:Arc<Mutex<Vec<(SocketAddr, TcpStream)>>>, contact:(SocketAddr, TcpStream)){
    let mut lock = contact_list.lock().unwrap();
    lock.push(contact);
}

pub fn contact_list_get_stream_by_socket_address(contact_list:Arc<Mutex<Vec<(SocketAddr, TcpStream)>>>, socket_address:SocketAddr)->Option<TcpStream>{
    let mut lock = contact_list.lock().unwrap();
    lock.sort_by_key(|k|k.0);
    match lock.binary_search_by_key(&socket_address, |k|k.0){
        Ok(i)=>{
            Some(lock.get(i).unwrap().1.try_clone().unwrap())
        }
        Err(_)=>{
            None
        }
    }
}

pub fn contact_list_display(contact_list:Arc<Mutex<Vec<(SocketAddr, TcpStream)>>>) {
    let lock = contact_list.lock().unwrap();
    lock
    .iter()
    .for_each(|x|println!("{}",x.0));
}