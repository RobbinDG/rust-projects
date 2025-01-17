use std::collections::HashMap;
use backend::protocol::new::queue_id::QueueId;
use crate::new::queue::Queue;

pub struct MessageQueue {
    queue: Queue,
}

pub struct MessageTopic {

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
        todo!()
    }

    pub fn create(&mut self, queue_id: QueueId) {
        todo!()
    }

    pub fn delete(&mut self, queue_id: &QueueId) {
        todo!()
    }

    pub fn get_queue_mut(&self, queue_id: &QueueId) -> Option<&mut Queue> {
        todo!()
    }
}