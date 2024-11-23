mod connection_manager;
mod request_handler;

use crate::connection_manager::ConnectionManager;
use backend::message::Message;
use backend::message_queue::MessageQueue;
use backend::request::{RequestError, ServerRequest};
use backend::response::ServerResponse;
use backend::status_code::Status;
use postcard::to_allocvec;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::str;
use std::sync::{Arc, Mutex};

pub struct QueueManager {
    queues: HashMap<String, MessageQueue>,
}

impl QueueManager {
    pub fn queues(&self) -> Vec<&String> {
        // TODO is a Vec a proper return type here?
        self.queues.keys().collect()
    }

    pub fn queue_exists(&self, name: &String) -> bool {
        self.queues.contains_key(name)
    }

    pub fn create(&mut self, name: String) {
        self.queues.insert(name, MessageQueue::new_empty());
    }

    pub fn push(&mut self, queue_name: &String, message: String) -> bool {
        if let Some(queue) = self.queues.get_mut(queue_name) {
            queue.put(Message::new(message));
            return true;
        }
        false
    }
}

pub struct RequestHandler {
    queue_manager: QueueManager,
}

impl RequestHandler {
    pub fn handle_request(&mut self, request: ServerRequest) -> Result<ServerResponse, RequestError> {
        match request {
            ServerRequest::ListQueues => {
                let queues_str: String = self.queue_manager.queues().iter().map(|s| s.as_str()).collect::<Vec<&str>>().join(" ,");
                println!("{:?}", queues_str);
                Ok(ServerResponse::from_str(queues_str.as_str()))
            }
            ServerRequest::CheckQueue(name) => {
                if self.queue_manager.queue_exists(&name) {
                    Ok(ServerResponse::from_status(Status::Exists))
                } else {
                    Ok(ServerResponse::from_status(Status::Failed))
                }
            }
            ServerRequest::CreateQueue(name) => {
                if self.queue_manager.queue_exists(&name) {
                    Ok(ServerResponse::from_status(Status::Exists))
                } else {
                    self.queue_manager.create(name);
                    Ok(ServerResponse::from_status(Status::Created))
                }
            }
            ServerRequest::PutMessage(queue_name, message) => {
                // TODO check queue exists
                if self.queue_manager.push(&queue_name, message) {
                    Ok(ServerResponse::from_status(Status::Sent))
                } else {
                    Ok(ServerResponse::from_status(Status::NotFound))
                }
            }
        }
    }
}

pub struct Server {
    // queue_manager: QueueManager,
    connection_manager: ConnectionManager,
}

impl Server {
    pub fn new(tcp_listener: TcpListener) -> Self {
        let mut queue_manager = QueueManager { queues: HashMap::default() };
        let request_handler = RequestHandler { queue_manager };
        let connection_manager = ConnectionManager::new(tcp_listener, request_handler);
        Self {
            // queue_manager,
            connection_manager,
        }
    }

    pub fn run(self) {
        self.connection_manager.start()
    }
}

fn main() {
    let socket_listener = TcpListener::bind("localhost:1234").unwrap();
    let mut server = Server::new(socket_listener);
    server.run();

    // loop {
    //     match socket_listener.accept() {
    //         Ok((mut _socket, addr)) => {
    //             println!("new client: {addr:?}");
    //
    //             loop {
    //                 if let Err(e) = execute_request(&mut server, &mut _socket) {
    //                     match e {
    //                         RequestError::IO(_) => {
    //                             println!("Client {addr:?} disconnected.")
    //                         }
    //                         RequestError::Parsing(err) => {
    //                             println!("{:?}", err);
    //                         }
    //                         RequestError::Internal(err) => {
    //                             println!("Internal Error: {:?}", err);
    //                         }
    //                     }
    //                     break;
    //                 }
    //             }
    //         }
    //         Err(e) => println!("couldn't get client: {e:?}"),
    //     };
    // }
}

// fn execute_request(server: &mut Server, _socket: &mut TcpStream) -> Result<(), RequestError> {
//     let mut buf = [0; 32];
//     _socket.read(&mut buf)?;
//     let request: Result<ServerRequest, postcard::Error> = postcard::from_bytes(&buf);
//     println!("Received {:?}", request);
//
//     _socket.flush()?;
//
//     let response = match request {
//         Ok(r) => server.handle_request(r)?,
//         Err(e) => {
//             println!("{:?}", e);
//             ServerResponse::from_status(Status::UnknownCommand)
//         }
//     };
//
//     let payload = to_allocvec(&response).unwrap();
//     _socket.write_all(&payload)?;
//     println!("written");
//     Ok(())
// }
