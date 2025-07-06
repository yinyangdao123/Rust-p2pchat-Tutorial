fn main(){
    let s = "abc 123";
    let a = s.split_once(char::is_whitespace);
    println!("{:?}",a);

    let s = "abc123";
    let a = s.split_once(char::is_whitespace);
    println!("{:?}",a);

    let s = " abc123";
    let a = s.split_once(char::is_whitespace);
    println!("{:?}",a);

    let s = "abc123 ";
    let a = s.split_once(char::is_whitespace);
    println!("{:?}",a);
}