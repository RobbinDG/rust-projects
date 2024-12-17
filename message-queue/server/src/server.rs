use crate::buffer_manager2::BufferManager;
use crate::connection_manager::ConnectionManager;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use crate::buffer_manager::BufferInterface;

pub struct Server {
    buffer_manager: Arc<Mutex<BufferManager>>,
    connection_manager: ConnectionManager,
}

impl Server {
    pub fn new(tcp_listener: TcpListener) -> Self {
        let buffer_manager = Arc::new(Mutex::new(BufferManager::new()));
        let connection_manager =
            ConnectionManager::new(tcp_listener, buffer_manager.clone());
        Self {
            buffer_manager,
            connection_manager,
        }
    }

    pub fn run(self) {
        let cm = Arc::new(self.connection_manager);
        let cm1 = cm.clone();
        thread::spawn(move || loop {
            {
                self.buffer_manager.lock().unwrap().process_queues();
            }
            cm1.check_and_join_disconnects().unwrap();

            thread::sleep(Duration::from_secs(1));
        });
        cm.start()
    }
}