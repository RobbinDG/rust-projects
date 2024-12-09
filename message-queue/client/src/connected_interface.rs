use crate::admin_interface::AdminInterface;
use crate::disconnected_interface::DisconnectedInterface;
use crate::interface::Interface;
use backend::message::Message;
use backend::request::SetModeResponse;
use backend::setup_request::SetupRequest;
use backend::ConnectedClient;
use std::io;

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
        println!(" [1] Make Admin");
        println!(" [2] Select queue");
        println!(" [4] Send messages");
        println!(" [5] Listen to queue");
        println!(" [0] Disconnect");
    }

    fn on_selection(mut self: Box<Self>, choice: u32) -> Box<dyn Interface> {
        match choice {
            1 => {
                match self.server.transfer_request(SetupRequest::Admin) {
                    Ok(response) => {
                        println!("Response {:?}", response);
                        if let SetModeResponse::Admin = response {
                            return Box::new(AdminInterface::new(self.server, self.selected_queue));
                        }
                    }
                    Err(err) => {
                        println!("Error during request {:?}", err)
                    }
                }
                Box::new(ConnectedInterface {
                    server: self.server,
                    selected_queue: self.selected_queue,
                })
            }
            2 => {
                let selection = prompt_string_input("Which queue do you want select?");
                Box::new(ConnectedInterface {
                    server: self.server,
                    selected_queue: Some(selection),
                })
            }
            4 => {
                if let Some(queue) = &self.selected_queue {
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
                    selected_queue: self.selected_queue,
                })
            }
            5 => {
                if let Some(queue) = &self.selected_queue {
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
