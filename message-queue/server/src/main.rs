mod connection_manager;
mod setup_worker;
mod request_handler;
mod queue_manager;
mod stream_worker;
mod admin_worker;
pub mod message_queue;

use crate::connection_manager::ConnectionManager;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::thread;
use queue_manager::QueueManager;

pub struct Server {
    queue_manager: Arc<Mutex<QueueManager>>,
    connection_manager: ConnectionManager,
}

impl Server {
    pub fn new(tcp_listener: TcpListener) -> Self {
        let queue_manager = Arc::new(Mutex::new(QueueManager::new()));
        let connection_manager =
            ConnectionManager::new(tcp_listener, queue_manager.clone());
        Self {
            queue_manager,
            connection_manager,
        }
    }

    pub fn run(self) {
        let mut cm = Arc::new(self.connection_manager);
        let cm1 = cm.clone();
        thread::spawn(move || loop {
            {
                self.queue_manager.lock().unwrap().process_queues();
            }
            cm1.check_and_join_disconnects().unwrap();

            thread::sleep(Duration::from_secs(1));
        });
        cm.start()
    }
}

fn main() {
    let socket_listener = TcpListener::bind("localhost:1234").unwrap();
    let server = Server::new(socket_listener);
    server.run();
}
