use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpListener;
use backend::response::ServerResponse;
use std::str;
use std::str::FromStr;
use backend::message::Message;
use backend::message_queue::MessageQueue;
use backend::request::ServerRequest;

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
    let mut server = Server { queues: HashMap::default() };

    let socket_listener = TcpListener::bind("localhost:1234").unwrap();

    match socket_listener.accept() {
        Ok((mut _socket, addr)) => {
            println!("new client: {addr:?}");

            loop {
                let mut buf = [0; 32];
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
                    Ok(ServerRequest::CheckQueue(name)) => {
                        let response;
                        if server.queues().contains(&&name) {
                            response = ServerResponse::from_str("exists");
                        } else {
                            response = ServerResponse::from_str("not_found");
                        }
                        _socket.write(response.as_payload().as_bytes()).unwrap();
                    }
                    Ok(ServerRequest::CreateQueue(name)) => {
                        let response;
                        if server.queues.contains_key(&name) {
                            response = ServerResponse::from_str("already_exists");
                        } else {
                            server.queues.insert(name, MessageQueue::new_empty());
                            response = ServerResponse::from_str("created");
                        }
                        _socket.write(response.as_payload().as_bytes()).unwrap();
                    }
                    Ok(ServerRequest::PutMessage(queue_name, message)) => {
                        // TODO check queue exists
                        let mut queue = server.queues.get_mut(&queue_name).unwrap();
                        queue.put(Message::new(message));
                        let response = ServerResponse::from_str("sent");
                        _socket.write(response.as_payload().as_bytes()).unwrap();
                    }
                    Err(e) => {
                        println!("{:?}", e);
                        _socket.write(b"not_understood").unwrap();
                    }
                }
            }
        }
        Err(e) => println!("couldn't get client: {e:?}"),
    };
}
