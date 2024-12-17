use crate::buffer_interface::BufferInterface;
use crate::buffer_processor::MessageQueueProcessor;
use crate::queue_manager::QueueManager;
use crate::topic_manager::TopicManager;
use crate::topic_processor::TopicProcessor;
use backend::protocol::{BufferAddress, BufferType};
use backend::stream_io::StreamIO;
use std::io;
use std::net::TcpStream;

pub struct BufferManager {
    queues: QueueManager,
    topics: TopicManager,
}

impl BufferManager {
    pub fn new() -> Self {
        Self {
            queues: QueueManager::new(MessageQueueProcessor {}),
            topics: TopicManager::new(TopicProcessor {}),
        }
    }
}

impl BufferInterface<BufferAddress> for BufferManager {
    fn buffers(&self) -> Vec<(BufferAddress, usize, usize, usize)> {
        let mut result = self.queues.buffers();
        result.extend(self.topics.buffers());
        result
    }

    fn queue_exists(&self, queue: &BufferAddress) -> bool {
        match queue.buffer_type() {
            // TODO check if to_string doesn't needlessly copy here
            BufferType::Queue => self.queues.queue_exists(&queue.to_string()),
            BufferType::Topic => self.topics.queue_exists(&queue.to_string()),
        }
    }

    fn create(&mut self, queue: BufferAddress) {
        match queue.buffer_type() {
            BufferType::Queue => self.queues.create(queue.to_string()),
            BufferType::Topic => self.topics.create(queue.to_string()),
        }
    }

    fn delete(&mut self, queue: &BufferAddress) -> Option<(Vec<StreamIO>, Vec<StreamIO>)> {
        match queue.buffer_type() {
            BufferType::Queue => self.queues.delete(&queue.to_string()),
            BufferType::Topic => self.topics.delete(&queue.to_string()),
        }
    }

    fn connect_sender(&mut self, queue: &BufferAddress, stream: TcpStream) -> io::Result<()> {
        match queue.buffer_type() {
            BufferType::Queue => self.queues.connect_sender(&queue.to_string(), stream),
            BufferType::Topic => self.topics.connect_sender(&queue.to_string(), stream),
        }
    }

    fn connect_receiver(&mut self, queue: &BufferAddress, stream: TcpStream) {
        match queue.buffer_type() {
            BufferType::Queue => self.queues.connect_receiver(&queue.to_string(), stream),
            BufferType::Topic => self.topics.connect_receiver(&queue.to_string(), stream),
        }
    }

    fn process_queues(&mut self) {
        self.queues.process_queues();
        self.topics.process_queues();
    }
}
