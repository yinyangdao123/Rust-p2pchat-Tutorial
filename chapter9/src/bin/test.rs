
use std::io::{self, Write};

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();

    println!("> |");
    stdout.flush()?;

    print!("abcdefg");
    print!("\x1B[2m{}", "ABCDEFG");
    print!("\x08 \x08");
    print!("\x1B[0m");
    print!("{}", "ABCDEFG");
    print!("\x1B[1m{}", "ABCDEFG");
    //print!("\x1B[1G\x1B[2K");
    stdout.flush()?;

    Ok(())
}
