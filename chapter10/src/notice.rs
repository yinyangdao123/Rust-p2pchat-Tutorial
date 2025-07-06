use std::io::Write;

pub enum Level {
    System,
    User,
    Info
}

pub fn notice_and_prompt(notice:Option<&str>, prompt:Option<&str>, level:Level){
    let color = match level { //0 system 1 user 2 info
        Level::System=>"\x1B[2m",
        Level::User=>"\x1B[32m",
        Level::Info=>"\x1B[33m"
    };

    if let Some(notice) = notice {
        println!("\x1B[1G\x1B[2K{}{}\x1B[0m", color, notice);
    }

    if let Some(prompt) = prompt {
        print!("{} ", prompt);
    }
    
    std::io::stdout().flush().unwrap();
}
