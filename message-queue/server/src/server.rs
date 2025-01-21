use crate::connection_manager::ConnectionManager;
use crate::new::queue_store::QueueStore;
use std::error::Error;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;

pub struct Server {
    connection_manager: ConnectionManager,
    queues: Arc<Mutex<QueueStore>>,
}

impl Server {
    pub fn new(tcp_listener: TcpListener) -> Self {
        let queues = Arc::new(Mutex::new(QueueStore::new()));
        let connection_manager = ConnectionManager::new(tcp_listener, queues.clone());
        Self {
            connection_manager,
            queues,
        }
    }

    pub async fn run(mut self) -> Result<(), Box<dyn Error>> {
        let cm = Arc::new(self.connection_manager);
        // let cm1 = cm.clone();
        // thread::spawn(move || loop {
        //     {
        //         self.buffer_manager.lock().unwrap().process_queues();
        //     }
        //     cm1.check_and_join_disconnects().unwrap();
        //
        //     thread::sleep(Duration::from_secs(1));
        // });
        Ok(cm.start().await)
        // loop {
        //     self.publisher_manager.receive(&mut self.router);
        //     self.subscription_manager.distribute(&mut self.queues);
        //     self.queue_manager.process_queues(&mut self.queues);
        //     thread::sleep(Duration::from_secs(1));
        // }
    }
}
