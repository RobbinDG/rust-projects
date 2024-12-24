use crate::admin_interface::AdminInterface;
use crate::disconnected_interface::DisconnectedInterface;
use crate::interface::Interface;
use backend::protocol::{BufferAddress, Message};
use backend::protocol::{SetupRequest, SetupResponse};
use backend::ConnectedClient;
use lipsum::lipsum;
use rand::prelude::*;
use std::io;
use std::thread::sleep;
use std::time::Duration;

pub fn prompt_string_input(prompt: &str) -> String {
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
    selected: Option<BufferAddress>,
}

impl ConnectedInterface {
    pub fn new(server: ConnectedClient<&'static str>) -> Self {
        Self {
            server,
            selected: None,
        }
    }
}

impl Interface for ConnectedInterface {
    fn print_options(&self) {
        println!(" [1] Make Admin");
        println!(" [2] Select queue");
        println!(" [3] Send messages");
        println!(" [4] Listen to queue");
        println!(" [0] Disconnect");
    }

    fn on_selection(mut self: Box<Self>, choice: u32) -> Box<dyn Interface> {
        match choice {
            1 => {
                match self.server.transfer_request(SetupRequest::Admin) {
                    Ok(response) => {
                        println!("Response {:?}", response);
                        if let SetupResponse::Admin = response {
                            return Box::new(AdminInterface::new(self.server, self.selected));
                        }
                    }
                    Err(err) => {
                        println!("Error during request {:?}", err)
                    }
                }
                Box::new(ConnectedInterface {
                    server: self.server,
                    selected: self.selected,
                })
            }
            2 => {
                let selection = prompt_string_input("Which queue do you want select?");
                println!("{:?}", BufferAddress::new(selection.clone()));
                Box::new(ConnectedInterface {
                    server: self.server,
                    selected: Some(BufferAddress::new(selection)),
                })
            }
            3 => {
                if let Some(queue) = &self.selected {
                    let response = self
                        .server
                        .transfer_request(SetupRequest::Sender(queue.clone()));
                    match response {
                        Ok(r) => println!("Response {:?}", r),
                        Err(e) => println!("Error {:?}", e),
                    }
                    self.send_messages();
                } else {
                    println!("No queue selected.");
                }
                Box::new(ConnectedInterface {
                    server: self.server,
                    selected: self.selected,
                })
            }
            4 => {
                if let Some(queue) = &self.selected {
                    println!("Started listening...");
                    let response = self
                        .server
                        .transfer_request(SetupRequest::Receiver(queue.clone()))
                        .unwrap();
                    println!("Response {:?}", response);
                    self.receive_messages();
                } else {
                    println!("No queue selected.");
                }
                Box::new(ConnectedInterface {
                    server: self.server,
                    selected: self.selected,
                })
            }
            0 => Box::new(DisconnectedInterface::new(self.server.disconnect())),
            _ => Box::new(ConnectedInterface {
                server: self.server,
                selected: self.selected,
            }),
        }
    }
}

impl ConnectedInterface {
    fn send_messages(&mut self) {
        loop {
            // let message_str = prompt_string_input("Write message");
            sleep(Duration::from_millis(rand::random_range(1000..3000)));
            let words = lipsum(5);
            self.server.push_message(Message::new(words)).unwrap();
        }
    }

    fn receive_messages(&mut self) {
        loop {
            let message: Message = self.server.pull_message().unwrap();
            println!("{:?}", message);
        }
    }
}
