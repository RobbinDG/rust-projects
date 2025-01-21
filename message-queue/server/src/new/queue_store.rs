use crate::new::queue::Queue;
use backend::protocol::new::message::Message;
use backend::protocol::new::queue_id::QueueId;
use std::collections::HashMap;

pub struct MessageQueue {
    queue: Queue,
}

impl MessageQueue {
    pub fn new() -> Self {
        Self {
            queue: Queue::new(),
        }
    }

    pub fn publish(&mut self, message: Message) {
        self.queue.push(message);
    }

    pub fn receive(&mut self) -> Option<Message> {
        self.queue.pop()
    }
}

pub struct MessageTopic {}

impl MessageTopic {
    pub fn new() -> Self {
        Self {}
    }

    pub fn publish(&mut self, message: Message) {}

    pub fn receive(&mut self) -> Option<Message> {
        None
    }
}

enum QueueType {
    Queue(MessageQueue),
    Topic(MessageTopic),
}

pub struct QueueStore {
    queues: HashMap<QueueId, QueueType>,
}

pub struct Publisher<'a> {
    queue: &'a mut QueueType,
}

impl<'a> Publisher<'a> {
    pub fn publish(&mut self, message: Message) {
        match self.queue {
            QueueType::Queue(q) => q.publish(message),
            QueueType::Topic(q) => q.publish(message),
        }
    }
}

pub struct Receiver<'a> {
    queue: &'a mut QueueType,
}

impl<'a> Receiver<'a> {
    pub fn receive(&mut self) -> Option<Message> {
        match self.queue {
            QueueType::Queue(q) => q.receive(),
            QueueType::Topic(t) => t.receive(),
        }
    }
}

impl QueueStore {
    pub fn new() -> Self {
        Self {
            queues: HashMap::new(),
        }
    }

    pub fn list(&self) -> Vec<(QueueId, usize, usize, usize)> {
        self.queues
            .keys()
            .cloned()
            .map(|id| (id, 0, 0, 0))
            .collect()
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

    pub fn exists(&self, queue_id: &QueueId) -> bool {
        self.queues.contains_key(queue_id)
    }

    pub fn delete(&mut self, queue_id: &QueueId) -> bool {
        self.queues.remove(queue_id).is_some()
    }

    pub fn publisher(&mut self, for_queue: &QueueId) -> Option<Publisher> {
        match self.queues.get_mut(for_queue) {
            None => None,
            Some(queue) => Some(Publisher { queue }),
        }
    }

    pub fn receiver(&mut self, for_queue: &QueueId) -> Option<Receiver> {
        match self.queues.get_mut(for_queue) {
            None => None,
            Some(queue) => Some(Receiver { queue }),
        }
    }
}
