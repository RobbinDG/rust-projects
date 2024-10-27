use std::collections::HashMap;
use std::io::Read;
use std::net::TcpListener;
use backend::MessageQueue;

pub struct Server {
    queues: HashMap<String, MessageQueue>,
}

impl Server {
    pub fn queues(&self) -> Vec<&String> {
        // TODO is a Vec a proper return type here?
        self.queues.keys().collect()
    }
}

fn main() {
    // println!("Hello, world!");
    let server = Server { queues: Default::default() };

    let socket_listener = TcpListener::bind("localhost:1234").unwrap();

    match socket_listener.accept() {
        Ok((mut _socket, addr)) => {
            println!("new client: {addr:?}");
            let mut buf = String::new();
            _socket.read_to_string(&mut buf).unwrap();
            println!("Received {:?}", buf);
        },
        Err(e) => println!("couldn't get client: {e:?}"),
    };
}
