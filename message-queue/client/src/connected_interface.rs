use std::io;
use backend::ConnectedClient;
use backend::request::ServerRequest;
use crate::disconnected_interface::DisconnectedInterface;
use crate::interface::Interface;


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

pub struct ConnectedInterface {
    server: ConnectedClient<&'static str>,
    selected_queue: Option<String>,
}

impl ConnectedInterface {
    pub fn new(server: ConnectedClient<&'static str>) -> Self {
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
        println!(" [5] Listen to queue");
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
                let name = prompt_string_input("Name your new queue...");
                let response = self.server.send_request(ServerRequest::CreateQueue(name)).unwrap();
                println!("Response {:?}", response);
                Box::new(ConnectedInterface { server: self.server, selected_queue: self.selected_queue })
            }
            5 => {
                if let Some(queue) = &self.selected_queue {
                    println!("Started listening...");
                    let response = self.server.send_request(ServerRequest::MakeReceiver(queue.clone())).unwrap();
                    println!("Response {:?}", response);
                } else {
                    println!("No queue selected.");
                }
                Box::new(ConnectedInterface { server: self.server, selected_queue: self.selected_queue })
            }
            0 => {
                Box::new(DisconnectedInterface::new(self.server.disconnect()))
            }
            _ => Box::new(ConnectedInterface { server: self.server, selected_queue: self.selected_queue })
        }
    }
}