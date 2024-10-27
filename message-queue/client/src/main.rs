use client::ServerConnection;
use std::io;
use std::io::Write;
use std::net::TcpStream;

fn main() {
    let mut stream = TcpStream::connect("localhost:1234").unwrap();

    match prompt_main() {
        1 => {
            stream.write("Test message".as_bytes()).unwrap();
            let conn = ServerConnection {};
            println!("{:?}", conn.available_queues());
        }
        _ => {},
    };
}

fn prompt_main() -> u32 {
    loop {
        println!("What to do?");
        println!(" [1] List queues");

        let mut buffer = String::new();
        if let Err(_) = io::stdin().read_line(&mut buffer) {
            continue;
        }

        match buffer[0..buffer.len() - 1].parse::<u32>() {
            Ok(val) => return val,
            Err(_) => { continue }
        }
    };
}