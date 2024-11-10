use backend::message::Message;
use backend::message_queue::MessageQueue;
use backend::request::{RequestError, ServerRequest};
use backend::response::ServerResponse;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::str;
use backend::status_code::Status;

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

    loop {
        match socket_listener.accept() {
            Ok((mut _socket, addr)) => {
                println!("new client: {addr:?}");

                loop {
                    if let Err(e) = execute_request(&mut server, &mut _socket) {
                        match e {
                            RequestError::IO(_) => {
                                println!("Client {addr:?} disconnected.")
                            }
                            RequestError::Parsing(err) => {
                                println!("{:?}", err);
                            }
                            RequestError::Internal(err) => {
                                println!("Internal Error: {:?}", err);
                            }
                        }
                        break;
                    }
                }
            }
            Err(e) => println!("couldn't get client: {e:?}"),
        };
    }
}

fn execute_request(server: &mut Server, _socket: &mut TcpStream) -> Result<(), RequestError> {
    let mut buf = [0; 32];
    _socket.read(&mut buf)?;
    let request_str = str::from_utf8(&buf)?.trim_matches('\0');
    let request = ServerRequest::parse(request_str);
    println!("Received {:?}", request_str);

    _socket.flush()?;
    let response: ServerResponse = match request {
        Ok(ServerRequest::ListQueues) => {
            let queues_str: String = server.queues().iter().map(|s| s.as_str()).collect::<Vec<&str>>().join(" ,");
            println!("{:?}", queues_str);
            ServerResponse::from_str(queues_str.as_str())
        }
        Ok(ServerRequest::CheckQueue(name)) => {
            if server.queues().contains(&&name) {
                ServerResponse::from_status(Status::Exists)
            } else {
                ServerResponse::from_status(Status::Failed)
            }
        }
        Ok(ServerRequest::CreateQueue(name)) => {
            if server.queues.contains_key(&name) {
                ServerResponse::from_status(Status::Exists)
            } else {
                server.queues.insert(name, MessageQueue::new_empty());
                ServerResponse::from_status(Status::Created)
            }
        }
        Ok(ServerRequest::PutMessage(queue_name, message)) => {
            // TODO check queue exists
            if let Some(queue) = server.queues.get_mut(&queue_name) {
                queue.put(Message::new(message));
                ServerResponse::from_status(Status::Sent)
            } else {
                ServerResponse::from_status(Status::NotFound)
            }
        }
        Err(e) => {
            println!("{:?}", e);
            ServerResponse::from_status(Status::UnknownCommand)
        }
    };
    _socket.write(response.as_payload().as_bytes())?;
    println!("written");
    Ok(())
}
