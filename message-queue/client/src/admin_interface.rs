use crate::disconnected_interface::DisconnectedInterface;
use crate::interface::Interface;
use backend::protocol::request::{CheckQueue, CreateQueue, ListQueues};
use backend::protocol::Status;
use backend::ConnectedClient;

pub struct AdminInterface {
    server: ConnectedClient<&'static str>,
    selected_queue: Option<String>
}

impl AdminInterface {
    pub fn new(server: ConnectedClient<&'static str>, selected_queue: Option<String>) -> Self {
        Self {
            server,
            selected_queue,
        }
    }
}

impl Interface for AdminInterface {
    fn print_options(&self) {
        println!(" [1] List queues");
        println!(" [2] Select queue");
        println!(" [3] Create queue");
    }

    fn on_selection(mut self: Box<Self>, choice: u32) -> Box<dyn Interface> {
        match choice {
            1 => {
                let response = self.server.transfer_admin_request(ListQueues {}).unwrap();
                println!("Response {:?}", response);
                Box::new(AdminInterface {
                    server: self.server,
                    selected_queue: self.selected_queue,
                })
            }
            2 => {
                let selection = crate::connected_interface::prompt_string_input("Which queue do you want select?");
                let response = self
                    .server
                    .transfer_admin_request(CheckQueue {
                        queue_address: selection.clone(),
                    })
                    .unwrap();
                if let Status::Exists = response {
                    // TODO replace with proper status code check
                    self.selected_queue = Some(selection);
                }
                println!("Response {:?}", response);
                Box::new(AdminInterface {
                    server: self.server,
                    selected_queue: self.selected_queue,
                })
            }
            3 => {
                let name = crate::connected_interface::prompt_string_input("Name your new queue...");
                let response = self
                    .server
                    .transfer_admin_request(CreateQueue { queue_address: name })
                    .unwrap();
                println!("Response {:?}", response);
                Box::new(AdminInterface {
                    server: self.server,
                    selected_queue: self.selected_queue,
                })
            }
            0 => Box::new(DisconnectedInterface::new(self.server.disconnect())),
            _ => Box::new(AdminInterface {
                server: self.server,
                selected_queue: self.selected_queue,
            }),
        }
    }
}