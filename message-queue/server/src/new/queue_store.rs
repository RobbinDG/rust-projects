use crate::new::queue::Queue;
use backend::protocol::new::queue_id::QueueId;
use std::collections::HashMap;

pub struct MessageQueue {
    queue: Queue,
}

impl MessageQueue {
    pub fn new() -> Self {
        todo!()
    }
}

pub struct MessageTopic {}

impl MessageTopic {
    pub fn new() -> Self {
        todo!()
    }
}

enum QueueType {
    Queue(MessageQueue),
    Topic(MessageTopic),
}

pub struct QueueStore {
    queues: HashMap<QueueId, QueueType>,
}

impl QueueStore {
    pub fn new() -> Self {
        Self {
            queues: HashMap::new(),
        }
    }

    pub fn list(&self) -> Vec<(QueueId, usize, usize, usize)> {
        self.queues.keys().cloned().map(|id| (id, 0, 0, 0)).collect()
    }

    pub fn create(&mut self, queue_id: QueueId) {
        match &queue_id {
            QueueId::Queue(name) => self
                .queues
                .insert(queue_id, QueueType::Queue(MessageQueue::new())),
            QueueId::Topic(name) => self
                .queues
                .insert(queue_id, QueueType::Topic(MessageTopic::new())),
        };
    }

    pub fn delete(&mut self, queue_id: &QueueId) {
        todo!()
    }

    pub fn get_queue_mut(&self, queue_id: &QueueId) -> Option<&mut Queue> {
        todo!()
    }
}
