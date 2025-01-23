use crate::queue::{DequeuedMessage, Queue};
use backend::protocol::message::Message;
use backend::protocol::queue_id::QueueId;
use backend::protocol::QueueProperties;
use std::collections::HashMap;

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
}

pub struct MessageTopic {
    properties: QueueProperties,
}

impl MessageTopic {
    pub fn new(properties: QueueProperties) -> Self {
        Self { properties }
    }

    pub fn publish(&mut self, message: Message) {}

    pub fn receive(&mut self) -> Option<DequeuedMessage> {
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
    pub fn receive(&mut self) -> Option<DequeuedMessage> {
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

    pub fn create(&mut self, queue_id: QueueId, properties: QueueProperties) {
        match &queue_id {
            QueueId::Queue(_) => self
                .queues
                .insert(queue_id, QueueType::Queue(MessageQueue::new(properties))),
            QueueId::Topic(_) => self
                .queues
                .insert(queue_id, QueueType::Topic(MessageTopic::new(properties))),
        };
    }

    pub fn exists(&self, queue_id: &QueueId) -> bool {
        self.queues.contains_key(queue_id)
    }

    pub fn properties(&self, queue_id: &QueueId) -> Option<&QueueProperties> {
        Some(match self.queues.get(queue_id)? {
            QueueType::Queue(q) => &q.properties,
            QueueType::Topic(t) => &t.properties,
        })
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
