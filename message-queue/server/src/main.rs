mod connection_manager;
mod request_handler;
mod connection_worker;

use crate::connection_manager::ConnectionManager;
use backend::message_queue::MessageQueue;
use request_handler::RequestHandler;
use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

pub struct QueueManager {
    queues: HashMap<String, (Vec<TcpStream>, MessageQueue, Vec<TcpStream>)>,
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
        self.queues.insert(name, (Vec::default(), MessageQueue::new_empty(), Vec::default()));
    }

    pub fn process_queues(&mut self) {
        for (name, (senders, queue, receivers)) in self.queues.iter() {}
    }

    pub fn connect_sender(&mut self, queue_name: &String, stream: TcpStream) {
        if let Some((senders, _, _)) = self.queues.get_mut(queue_name) {
            senders.push(stream);
        }
    }

    pub fn connect_receiver(&mut self, queue_name: &String, stream: TcpStream) {
        if let Some((_, _, recipients)) = self.queues.get_mut(queue_name) {
            recipients.push(stream);
        }
    }
}

pub struct Server {
    // queue_manager: QueueManager,
    connection_manager: ConnectionManager,
}

impl Server {
    pub fn new(tcp_listener: TcpListener) -> Self {
        let queue_manager = Arc::new(Mutex::new(QueueManager { queues: HashMap::default() }));
        let request_handler = RequestHandler::new(queue_manager);
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
    let server = Server::new(socket_listener);
    server.run();
}
