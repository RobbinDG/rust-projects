use backend::DisconnectedClient;
use crate::connected_interface::ConnectedInterface;
use crate::interface::Interface;

pub struct DisconnectedInterface {
    server: DisconnectedClient<&'static str>,
}

impl DisconnectedInterface {
    pub fn new(server: DisconnectedClient<&'static str>) -> Self {
        Self { server }
    }
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