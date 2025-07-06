use std::{fs::File, io::{BufRead, BufReader, Read}};


fn main() {


    let file = File::open("./data.txt").unwrap();
    let mut file = BufReader::new(file);


    let mut buffer_head:[u8;1] = [0;1];
    file.read_exact(&mut buffer_head).unwrap();

    match str::from_utf8(&buffer_head) {
        Ok("0")=>{
            println!("we got 0.");
            let mut buffer_head_line = String::new();
            let count = file.read_line(&mut buffer_head_line).unwrap();
            println!("{}\n{}", count, buffer_head_line);
            

            let mut buffer_file = [0; 1024];
            let mut received:Vec<u8> = Vec::new();
            let mut total_received = 0;
            
            loop {
                println!("=================================start===========================================");
                let count = file.read(&mut buffer_file).unwrap();
                println!("{}\n{:?}", count, &buffer_file[..count]);

                if count==0 {break;}
                total_received = total_received + count;
                received.append(&mut buffer_file[..count].to_vec());
            }

            println!("=================================finally===========================================");
            println!("{}\n{}", total_received, str::from_utf8(&received).unwrap());
        }
        _=>{}
    }

    




}