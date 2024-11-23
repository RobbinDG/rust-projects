mod connection_manager;
mod request_handler;
mod connection_worker;

use crate::connection_manager::ConnectionManager;
use backend::message::Message;
use backend::message_queue::MessageQueue;
use std::collections::HashMap;
use std::net::TcpListener;
use request_handler::RequestHandler;

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

pub struct Server {
    // queue_manager: QueueManager,
    connection_manager: ConnectionManager,
}

impl Server {
    pub fn new(tcp_listener: TcpListener) -> Self {
        let queue_manager = QueueManager { queues: HashMap::default() };
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
