use crate::disconnected_interface::DisconnectedInterface;
use crate::interface::Interface;
use backend::message::Message;
use backend::request::{
    CheckQueue, CreateQueue, ListQueues, MakeReceiver, MakeSender,
};
use backend::ConnectedClient;
use std::io;
use backend::status_code::Status;

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
        println!(" [3] Create queue");
        println!(" [4] Send messages");
        println!(" [5] Listen to queue");
        println!(" [0] Disconnect");
    }

    fn on_selection(mut self: Box<Self>, choice: u32) -> Box<dyn Interface> {
        match choice {
            1 => {
                let response = self.server.transfer_request(ListQueues {}).unwrap();
                println!("Response {:?}", response);
                Box::new(ConnectedInterface {
                    server: self.server,
                    selected_queue: self.selected_queue,
                })
            }
            2 => {
                let selection = prompt_string_input("Which queue do you want select?");
                let response = self
                    .server
                    .transfer_request(CheckQueue {
                        queue_name: selection.clone(),
                    })
                    .unwrap();
                if let Status::Exists = response {
                    // TODO replace with proper status code check
                    self.selected_queue = Some(selection);
                }
                println!("Response {:?}", response);
                Box::new(ConnectedInterface {
                    server: self.server,
                    selected_queue: self.selected_queue,
                })
            }
            3 => {
                let name = prompt_string_input("Name your new queue...");
                let response = self
                    .server
                    .transfer_request(CreateQueue { queue_name: name })
                    .unwrap();
                println!("Response {:?}", response);
                Box::new(ConnectedInterface {
                    server: self.server,
                    selected_queue: self.selected_queue,
                })
            }
            4 => {
                if let Some(queue) = &self.selected_queue {
                    let response = self
                        .server
                        .transfer_request(MakeSender {
                            destination_queue: queue.clone(),
                        })
                        .unwrap();
                    println!("Response {:?}", response);
                    self.send_messages();
                } else {
                    println!("No queue selected.");
                }
                Box::new(ConnectedInterface {
                    server: self.server,
                    selected_queue: self.selected_queue,
                })
            }
            5 => {
                if let Some(queue) = &self.selected_queue {
                    println!("Started listening...");
                    let response = self
                        .server
                        .transfer_request(MakeReceiver {
                            origin_queue: queue.clone(),
                        })
                        .unwrap();
                    println!("Response {:?}", response);
                    self.receive_messages();
                } else {
                    println!("No queue selected.");
                }
                Box::new(ConnectedInterface {
                    server: self.server,
                    selected_queue: self.selected_queue,
                })
            }
            0 => Box::new(DisconnectedInterface::new(self.server.disconnect())),
            _ => Box::new(ConnectedInterface {
                server: self.server,
                selected_queue: self.selected_queue,
            }),
        }
    }
}

impl ConnectedInterface {
    fn send_messages(&mut self) {
        loop {
            let message_str = prompt_string_input("Write message");
            self.server.send_message(Message::new(message_str)).unwrap();
        }
    }

    fn receive_messages(&mut self) {
        loop {
            let message = self.server.receive_message().unwrap();
            println!("{:?}", message);
        }
    }
}
