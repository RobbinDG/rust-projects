use crate::message_queue::MessageQueue;
use crate::message_topic::MessageTopic;
use crate::queue::DequeuedMessage;
use backend::protocol::client_id::ClientID;
use backend::protocol::message::Message;
use backend::protocol::queue_id::QueueId;
use backend::protocol::QueueProperties;
use std::collections::HashMap;

enum QueueType {
    Queue(MessageQueue),
    Topic(MessageTopic),
}

/// An interface to a set of queues to be managed. This object provides accessors and modifiers
/// to predictably handle everything one could do with a queue, whilst limiting the access
/// to the individual queues themselves to only reasonable actions.
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
    client: &'a ClientID,
    queue: &'a mut QueueType,
}

impl<'a> Receiver<'a> {
    pub fn receive(&mut self) -> Option<DequeuedMessage> {
        match self.queue {
            QueueType::Queue(q) => q.receive(),
            QueueType::Topic(t) => t.receive(self.client),
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
            QueueId::Topic(_, _, _) => self
                .queues
                .insert(queue_id, QueueType::Topic(MessageTopic::new(properties))),
        };
    }

    pub fn exists(&self, queue_id: &QueueId) -> bool {
        self.queues.contains_key(queue_id)
    }

    pub fn properties(&self, queue_id: &QueueId) -> Option<&QueueProperties> {
        Some(match self.queues.get(queue_id)? {
            QueueType::Queue(q) => q.properties(),
            QueueType::Topic(t) => t.properties(),
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

    pub fn receiver<'a>(
        &'a mut self,
        for_client: &'a ClientID,
        for_queue: &QueueId,
    ) -> Option<Receiver<'a>> {
        match self.queues.get_mut(for_queue) {
            None => None,
            Some(queue) => Some(Receiver {
                queue,
                client: for_client,
            }),
        }
    }

    /// Forwards resource allocation required for a client to receive messages from the provided
    /// queue. This method is not checked, so in poor use of this method might lead to large
    /// quantities of unused or under-utilised memory. The implementation is dependent
    /// on the target queue type. No allocation will be done if there exists no queue
    /// with the given identifier.
    ///
    /// # Arguments
    ///
    /// * `queue_id`: the queue to forward allocation for.
    /// * `client`: the client to allocate for.
    ///
    /// returns: `()`
    pub fn register_client(&mut self, queue_id: &QueueId, client: ClientID) {
        if let Some(queue) = self.queues.get_mut(queue_id) {
            match queue {
                QueueType::Queue(_) => {
                    // Not currently allocating anything for queues.
                }
                QueueType::Topic(t) => {
                    t.register_client(client);
                }
            }
        }
    }

    /// Forwards resource de-allocation; the inverse of `register_client()`.
    ///
    /// # Arguments
    ///
    /// * `queue_id`: the queue to forward allocation for.
    /// * `client`: the client to allocate for.
    ///
    /// returns: `()`
    pub fn deregister_client(&mut self, queue_id: &QueueId, client: &ClientID) {
        if let Some(queue) = self.queues.get_mut(queue_id) {
            match queue {
                QueueType::Queue(_) => {
                    // Not currently allocating anything for queues.
                }
                QueueType::Topic(t) => {
                    t.deregister_client(client);
                }
            }
        }
    }
}
