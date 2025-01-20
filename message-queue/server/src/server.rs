use crate::buffer_manager::BufferManager;
use crate::connection_manager::ConnectionManager;
use tokio::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use crate::buffer_interface::BufferInterface;
use crate::new::publisher_manager::PublisherManager;
use crate::new::queue_manager::QueueManager;
use crate::new::queue_store::QueueStore;
use crate::new::router::Router;
use crate::new::subscription_manager::SubscriptionManager;

pub struct Server {
    buffer_manager: Arc<Mutex<BufferManager>>,
    connection_manager: ConnectionManager,
    queues: QueueStore,
    router: Router,
    queue_manager: QueueManager,
    publisher_manager: PublisherManager,
    subscription_manager: SubscriptionManager,
}

impl Server {
    pub fn new(tcp_listener: TcpListener) -> Self {
        let buffer_manager = Arc::new(Mutex::new(BufferManager::new()));
        let connection_manager =
            ConnectionManager::new(tcp_listener, buffer_manager.clone());
        Self {
            buffer_manager,
            connection_manager,
            queue_manager: QueueManager::new(),
            publisher_manager: PublisherManager::new(),
            subscription_manager: SubscriptionManager::new(),
        }
    }

    pub fn run(mut self) {
        let cm = Arc::new(self.connection_manager);
        let cm1 = cm.clone();
        thread::spawn(move || loop {
            {
                self.buffer_manager.lock().unwrap().process_queues();
            }
            cm1.check_and_join_disconnects().unwrap();

            thread::sleep(Duration::from_secs(1));
        });
        cm.start();
        loop {
            self.publisher_manager.receive(&mut self.router);
            self.subscription_manager.distribute(&mut self.queues);
            self.queue_manager.process_queues(&mut self.queues);
            thread::sleep(Duration::from_secs(1));
        }
    }
}