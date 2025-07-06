use std::sync::{Arc, Mutex};

fn ad(xx: Arc<Mutex<i32>>){
    let mut l = xx.lock().unwrap();
    *l = 1;
}

fn main() {
    let x = Arc::new(Mutex::new(0));
    println!("{:?}", x);
    
    ad(Arc::clone(&x));

    println!("{:?}", x);
}
