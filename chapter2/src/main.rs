fn main() {
    let s = String::from("hello");
    match s.as_str() {
        "hello"=>println!("matched"),
        _=>println!("nomatch")
    }
}
