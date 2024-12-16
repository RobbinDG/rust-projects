use std::sync::{Arc, Mutex};
use std::net::TcpListener;
use std::thread;
use std::time::Duration;
use crate::buffer_processor::MessageQueueProcessor;
use crate::connection_manager::ConnectionManager;
use crate::queue_manager::QueueManager;
use crate::topic_manager::TopicManager;
use crate::topic_processor::TopicProcessor;

pub struct Server {
    queue_manager: Arc<Mutex<QueueManager>>,
    topic_manager: Arc<Mutex<TopicManager>>,
    connection_manager: ConnectionManager,
}

impl Server {
    pub fn new(tcp_listener: TcpListener) -> Self {
        let queue_manager = Arc::new(Mutex::new(QueueManager::new(MessageQueueProcessor {})));
        let topic_manager = Arc::new(Mutex::new(TopicManager::new(TopicProcessor {})));
        let connection_manager =
            ConnectionManager::new(tcp_listener, queue_manager.clone(), topic_manager.clone());
        Self {
            queue_manager,
            topic_manager,
            connection_manager,
        }
    }

    pub fn run(self) {
        let cm = Arc::new(self.connection_manager);
        let cm1 = cm.clone();
        thread::spawn(move || loop {
            {
                self.queue_manager.lock().unwrap().process_queues();
            }
            {
                self.topic_manager.lock().unwrap().process_queues()
            }
            cm1.check_and_join_disconnects().unwrap();

            thread::sleep(Duration::from_secs(1));
        });
        cm.start()
    }
}