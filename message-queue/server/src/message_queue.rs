use backend::protocol::message::Message;
use backend::protocol::QueueProperties;
use crate::queue::{DequeuedMessage, Queue};

pub struct MessageQueue {
    queue: Queue,
    properties: QueueProperties,
}

impl MessageQueue {
    pub fn new(properties: QueueProperties) -> Self {
        Self {
            queue: Queue::new(),
            properties,
        }
    }

    pub fn publish(&mut self, message: Message) {
        self.queue.push(message);
    }

    pub fn receive(&mut self) -> Option<DequeuedMessage> {
        self.queue.pop()
    }

    pub fn properties(&self) -> &QueueProperties {
        &self.properties
    }
}