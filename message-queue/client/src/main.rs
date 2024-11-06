use backend::DisconnectedServer;
use std::io;
use backend::request::ServerRequest;

fn main() {

    let mut server = DisconnectedServer::new();
    let mut connected_server = server.connect().unwrap();
    let mut selected_queue: Option<String> = None;

    loop {
    match prompt_main() {
        1 => {
            let response = connected_server.send_request(ServerRequest::ListQueues).unwrap();
            println!("Response {:?}", response);
        },
        2 => {
            let selection = prompt_string_input("Which queue do you want select?");
            let response = connected_server.send_request(ServerRequest::CheckQueue(selection.clone())).unwrap();
            if response.payload == "exists" { // TODO replace with proper status code check
                selected_queue = Some(selection);
            }
            println!("Response {:?}", response);
        }
        3 => {
            if let Some(queue) = &selected_queue {
                let message = prompt_string_input("Write your message?");
                let response = connected_server.send_request(ServerRequest::PutMessage(queue.clone(), message)).unwrap();
                println!("Response {:?}", response);
            } else {
                println!("No queue selected.");
            }
        }
        4 => {
            let name = prompt_string_input("Name your new queue?");
            let response = connected_server.send_request(ServerRequest::CreateQueue(name)).unwrap();
            println!("Response {:?}", response);
        }
        _ => {},
    };
        }
}

fn prompt_main() -> u32 {
    loop {
        println!("What to do?");
        println!(" [1] List queues");
        println!(" [2] Select queue");
        println!(" [3] X Send message");
        println!(" [4] Create queue");
        println!(" [5] Create echo listener");

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

fn prompt_string_input(prompt: &str) -> String {
    loop {
        let mut buffer = String::new();

        println!("{}", prompt);

        if let Err(_) = io::stdin().read_line(&mut buffer) {
            continue;
        }

        return buffer
    }
}
