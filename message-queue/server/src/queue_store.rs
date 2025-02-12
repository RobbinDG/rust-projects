use crate::message_queue::MessageQueue;
use crate::message_topic::MessageTopic;
use crate::queue::DequeuedMessage;
use backend::protocol::client_id::ClientID;
use backend::protocol::message::Message;
use backend::protocol::queue_id::{NewQueueId, QueueFilter, QueueId, TopLevelQueueId};
use backend::protocol::QueueProperties;
use std::collections::HashMap;

/// An interface to a set of queues to be managed. This object provides accessors and modifiers
/// to predictably handle everything one could do with a queue, whilst limiting the access
/// to the individual queues themselves to only reasonable actions.
pub struct QueueStore {
    directs: HashMap<String, MessageQueue>,
    primary_topics: HashMap<String, MessageTopic>,
}

pub struct QueuePublisher<'a> {
    queue: &'a mut MessageQueue,
}

impl<'a> QueuePublisher<'a> {
    fn publish(&'a mut self, message: Message) {
        self.queue.publish(message)
    }
}

pub struct TopicPublisher<'a> {
    topic: &'a mut MessageTopic,
    f1: String,
    f2: String,
}

impl<'a> TopicPublisher<'a> {
    fn publish(&'a mut self, message: Message) -> Result<(), Message> {
        self.topic
            .publish(message, self.f1.clone(), self.f2.clone())
    }
}

pub enum Publisher<'a> {
    Queue(QueuePublisher<'a>),
    Topic(TopicPublisher<'a>),
}

impl<'a> Publisher<'a> {
    /// Publishes a message to the designated queue. Returns the message if the queue was
    /// unable to receive it.
    ///
    /// # Arguments
    ///
    /// * `message`: the message to publish.
    ///
    /// returns: `Result<(), Message>`
    pub fn publish(&'a mut self, message: Message) -> Result<(), Message> {
        match self {
            Publisher::Queue(q) => Ok(q.publish(message)),
            Publisher::Topic(t) => t.publish(message),
        }
    }
}

pub struct QueueReceiver<'a> {
    queue: &'a mut MessageQueue,
}

impl<'a> QueueReceiver<'a> {
    fn receive(&mut self) -> Option<DequeuedMessage> {
        self.queue.receive()
    }
}

pub struct TopicReceiver<'a> {
    client: &'a ClientID,
    topic: &'a mut MessageTopic,
}

impl<'a> TopicReceiver<'a> {
    fn receive(&mut self) -> Option<DequeuedMessage> {
        self.topic.receive(self.client)
    }
}

pub enum Receiver<'a> {
    Queue(QueueReceiver<'a>),
    Topic(TopicReceiver<'a>),
}

impl Receiver<'_> {
    pub fn receive(&mut self) -> Option<DequeuedMessage> {
        match self {
            Receiver::Queue(q) => q.receive(),
            Receiver::Topic(t) => t.receive(),
        }
    }
}

impl QueueStore {
    pub fn new() -> Self {
        Self {
            directs: HashMap::new(),
            primary_topics: HashMap::new(),
        }
    }

    pub fn list(&self) -> Vec<TopLevelQueueId> {
        let mut result: Vec<TopLevelQueueId> = self
            .directs
            .keys()
            .cloned()
            .map(TopLevelQueueId::Queue)
            .collect();
        result.extend(
            self.primary_topics
                .keys()
                .cloned()
                .map(TopLevelQueueId::Topic),
        );
        result
    }

    pub fn message_count(&self, queue: &TopLevelQueueId) -> usize {
        match queue {
            TopLevelQueueId::Queue(q) => self
                .directs
                .get(q)
                .map(|queue| queue.message_count())
                .unwrap_or(0),
            TopLevelQueueId::Topic(t) => self
                .primary_topics
                .get(t)
                .map(|topic| topic.message_count())
                .unwrap_or(0),
        }
    }

    pub fn create(&mut self, queue_id: NewQueueId, properties: QueueProperties) -> bool {
        match queue_id {
            NewQueueId::Queue(name) => {
                self.directs.insert(name, MessageQueue::new(properties)).is_some()
            }
            NewQueueId::Topic(name, sub) => {
                let topic = self
                    .primary_topics
                    .entry(name)
                    .or_insert_with(|| MessageTopic::new(properties));
                if let Some(sub) = sub {
                    topic.create_subtopic(sub);
                    true
                } else {
                    false
                }
            }
        }
    }

    pub fn exists(&self, queue_id: &QueueId) -> bool {
        match queue_id {
            QueueId::Queue(name) => self.directs.contains_key(name),
            QueueId::Topic(name, f1, f2) => self
                .primary_topics
                .get(name)
                .map_or(false, |t| t.subtopic_exists((f1.clone(), f2.clone()))),
        }
    }

    pub fn is_filter_valid(&self, filter: &QueueFilter) -> bool {
        match filter {
            QueueFilter::Queue(name) => self.directs.contains_key(name),
            QueueFilter::Topic(name, f1, f2) => self
                .primary_topics
                .get(name)
                .map_or(false, |t| t.is_filter_valid((f1, f2))),
        }
    }

    pub fn get_topic(&self, name: &String) -> Option<&MessageTopic> {
        self.primary_topics.get(name)
    }

    pub fn properties(&self, queue_id: &TopLevelQueueId) -> Option<&QueueProperties> {
        match queue_id {
            TopLevelQueueId::Queue(name) => self.directs.get(name).map(|q| q.properties()),
            TopLevelQueueId::Topic(name) => self.primary_topics.get(name).map(|q| q.properties()),
        }
    }

    pub fn delete(&mut self, queue_id: &TopLevelQueueId) -> bool {
        match queue_id {
            TopLevelQueueId::Queue(name) => self.directs.remove(name).is_some(),
            TopLevelQueueId::Topic(name) => self.primary_topics.remove(name).is_some(),
        }
    }

    pub fn publisher(&mut self, for_queue: &QueueId) -> Option<Publisher> {
        match for_queue {
            QueueId::Queue(name) => self
                .directs
                .get_mut(name)
                .map(|queue| Publisher::Queue(QueuePublisher { queue })),
            QueueId::Topic(topic, f1, f2) => self.primary_topics.get_mut(topic).map(|topic| {
                Publisher::Topic(TopicPublisher {
                    topic,
                    f1: f1.clone(),
                    f2: f2.clone(),
                })
            }),
        }
    }

    pub fn receiver<'a>(
        &'a mut self,
        for_client: &'a ClientID,
        for_queue: &QueueFilter,
    ) -> Option<Receiver<'a>> {
        match for_queue {
            QueueFilter::Queue(name) => self
                .directs
                .get_mut(name)
                .map(|queue| Receiver::Queue(QueueReceiver { queue })),
            QueueFilter::Topic(topic, _, _) => self.primary_topics.get_mut(topic).map(|topic| {
                Receiver::Topic(TopicReceiver {
                    topic,
                    client: for_client,
                })
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
    pub fn register_client(&mut self, queue: &QueueFilter, client: ClientID) {
        match queue {
            QueueFilter::Queue(_) => {
                // Not currently allocating anything for queues.
            }
            QueueFilter::Topic(name, f1, f2) => {
                self.primary_topics
                    .get_mut(name)
                    .map(|topic| topic.register_client(client, (f1.clone(), f2.clone())));
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
    pub fn deregister_client(&mut self, queue: &QueueFilter, client: &ClientID) {
        match queue {
            QueueFilter::Queue(_) => {
                // Not currently allocating anything for queues.
            }
            QueueFilter::Topic(name, _, _) => {
                self.primary_topics
                    .get_mut(name)
                    .map(|topic| topic.deregister_client(client));
            }
        }
    }
}
