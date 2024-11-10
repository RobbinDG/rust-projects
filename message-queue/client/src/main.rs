use backend::request::ServerRequest;
use backend::{ConnectedServer, DisconnectedServer};
use std::io;

trait Interface {
    fn print_query(&self) {
        println!("What shall we do?");
        self.print_options();
    }
    fn print_options(&self);

    fn on_selection(self: Box<Self>, choice: u32) -> Box<dyn Interface>;
}

struct DisconnectedInterface {
    server: DisconnectedServer<&'static str>,
}

impl Interface for DisconnectedInterface {
    fn print_options(&self) {
        println!(" [0] Connect");
    }

    fn on_selection(self: Box<Self>, choice: u32) -> Box<dyn Interface> {
        match choice {
            0 => {
                match self.server.connect() {
                    Ok(server) => Box::new(ConnectedInterface::new(server)),
                    Err(e) => Box::new(DisconnectedInterface { server: e.server })
                }
            }
            _ => Box::new(DisconnectedInterface { server: self.server })
        }
    }
}

struct ConnectedInterface {
    server: ConnectedServer<&'static str>,
    selected_queue: Option<String>,
}

impl ConnectedInterface {
    pub fn new(server: ConnectedServer<&'static str>) -> Self {
        Self {
            server,
            selected_queue: None,
        }
    }
}

impl Interface for ConnectedInterface {
    fn print_options(&self) {
        println!(" [1] List queues");
        println!(" [2] Select queue");
        println!(" [3] X Send message");
        println!(" [4] Create queue");
        println!(" [5] Create echo listener");
        println!(" [0] Disconnect");
    }

    fn on_selection(mut self: Box<Self>, choice: u32) -> Box<dyn Interface> {
        match choice {
            1 => {
                let response = self.server.send_request(ServerRequest::ListQueues).unwrap();
                println!("Response {:?}", response);
                Box::new(ConnectedInterface { server: self.server, selected_queue: self.selected_queue })
            }
            2 => {
                let selection = prompt_string_input("Which queue do you want select?");
                let response = self.server.send_request(ServerRequest::CheckQueue(selection.clone())).unwrap();
                if response.payload == "exists" { // TODO replace with proper status code check
                    self.selected_queue = Some(selection);
                }
                println!("Response {:?}", response);
                Box::new(ConnectedInterface { server: self.server, selected_queue: self.selected_queue })
            }
            3 => {
                if let Some(queue) = &self.selected_queue {
                    let message = prompt_string_input("Write your message?");
                    let response = self.server.send_request(ServerRequest::PutMessage(queue.clone(), message)).unwrap();
                    println!("Response {:?}", response);
                } else {
                    println!("No queue selected.");
                }
                Box::new(ConnectedInterface { server: self.server, selected_queue: self.selected_queue })
            }
            4 => {
                let name = prompt_string_input("Name your new queue?");
                let response = self.server.send_request(ServerRequest::CreateQueue(name)).unwrap();
                println!("Response {:?}", response);
                Box::new(ConnectedInterface { server: self.server, selected_queue: self.selected_queue })
            }
            0 => {
                Box::new(DisconnectedInterface { server: self.server.disconnect() })
            }
            _ => Box::new(ConnectedInterface { server: self.server, selected_queue: self.selected_queue })
        }
    }
}

fn main() {
    let server = DisconnectedServer::new("localhost:1234");
    let mut interface: Box<dyn Interface> = Box::new(DisconnectedInterface { server });

    loop {
        interface.print_query();
        let choice = await_input();
        interface = interface.on_selection(choice);
    }
}

fn await_input() -> u32 {
    loop {
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

        return buffer;
    }
}
