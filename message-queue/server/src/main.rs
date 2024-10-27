use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpListener;
use backend::{MessageQueue, ServerRequest, ServerResponse};
use std::str;

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
    let server = Server { queues: HashMap::default() };

    let socket_listener = TcpListener::bind("localhost:1234").unwrap();

    match socket_listener.accept() {
        Ok((mut _socket, addr)) => {
            println!("new client: {addr:?}");

            loop {
                let mut buf = [0; 16];
                _socket.read(&mut buf).unwrap();
                let request_str = str::from_utf8(&buf).unwrap().trim_matches('\0');
                let request = ServerRequest::parse(request_str);
                println!("Received {:?}", request_str);

                _socket.flush().unwrap();

                match request {
                    Ok(ServerRequest::ListQueues) => {
                        let queues_str: String = server.queues().iter().map(|s| s.as_str()).collect::<Vec<&str>>().join(" ,");
                        println!("{:?}", queues_str);
                        let response = ServerResponse { payload: queues_str };
                        _socket.write(response.as_payload().as_bytes()).unwrap();
                        println!("written");
                    }
                    Err(e) => {
                        println!("{:?}", e);
                        _socket.write(b"not understood").unwrap();
                    }
                }
            }
        }
        Err(e) => println!("couldn't get client: {e:?}"),
    };
}
