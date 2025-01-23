use crate::connection_manager::ConnectionManager;
use crate::queue_store::QueueStore;
use std::error::Error;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;

pub struct Server {
    connection_manager: ConnectionManager,
}

impl Server {
    pub fn new(tcp_listener: TcpListener) -> Self {
        let queues = Arc::new(Mutex::new(QueueStore::new()));
        let connection_manager = ConnectionManager::new(tcp_listener, queues);
        Self { connection_manager }
    }

    pub async fn run(self) -> Result<(), Box<dyn Error>> {
        let cm = Arc::new(self.connection_manager);
        Ok(cm.start().await)
    }
}
