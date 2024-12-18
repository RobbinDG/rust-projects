mod interface;
mod connected_interface;
mod disconnected_interface;
mod admin_interface;

use backend::DisconnectedClient;
use std::io;
use crate::disconnected_interface::DisconnectedInterface;
use crate::interface::Interface;

fn main() {
    let server = DisconnectedClient::new("localhost:1234");
    let mut interface: Box<dyn Interface> = Box::new(DisconnectedInterface::new(server));

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