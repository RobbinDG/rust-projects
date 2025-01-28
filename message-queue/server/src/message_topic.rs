use crate::queue::{DequeuedMessage, Queue};
use crate::topic_filter_tree::TopicFilterTree;
use backend::protocol::client_id::ClientID;
use backend::protocol::message::Message;
use backend::protocol::queue_id::TopicLiteral;
use backend::protocol::QueueProperties;
use log::{debug, info};
use std::collections::{HashMap, HashSet};

pub struct MessageTopic {
    properties: QueueProperties,
    clients_by_filter: TopicFilterTree,
    client_queues: HashMap<ClientID, Queue>,
}

impl MessageTopic {
    pub fn new(properties: QueueProperties) -> Self {
        Self {
            properties,
            clients_by_filter: TopicFilterTree::new(),
            client_queues: HashMap::new(),
        }
    }

    pub fn get_subtopics(&self) -> HashMap<&String, HashSet<&String>> {
        self.clients_by_filter.subtopic_tree()
    }

    pub fn create_subtopic(&mut self, filter: (String, String)) {
        self.clients_by_filter.create_subtopic(filter.0.clone());
        self.clients_by_filter.create_subsubtopic(filter.0, filter.1)
    }

    pub fn subtopic_exists(&self, filter: (String, String)) -> bool {
        // TODO this is really inefficient (but quick)
        self.get_subtopics().get(&filter.0).and_then(|sub| sub.get(&filter.1)).is_some()
    }

    pub fn filter_valid(&self, filter: (&TopicLiteral, &TopicLiteral)) -> bool {
       self.clients_by_filter.filter_nonempty(filter)
    }

    pub fn receive(&mut self, client: &ClientID) -> Option<DequeuedMessage> {
        debug!("Receiving message for {:?}", client);
        self.client_queues
            .get_mut(client)
            .and_then(|queue| queue.pop())
    }

    pub fn register_client(
        &mut self,
        client: ClientID,
        topic_filter: (TopicLiteral, TopicLiteral),
    ) {
        info!("Creating topic buffer for {:?}", client);
        self.clients_by_filter.insert(client.clone(), topic_filter);
        self.client_queues.insert(client, Queue::new());
    }

    pub fn deregister_client(&mut self, client: &ClientID) {
        self.clients_by_filter.remove(client);
        self.client_queues.remove(client);
    }

    pub fn properties(&self) -> &QueueProperties {
        &self.properties
    }

    pub fn publish(&mut self, message: Message, f1: String, f2: String) {
        let clients = self.clients_by_filter.get_clients((f1, f2));
        for client in clients {
            if let Some(queue) = self.client_queues.get_mut(client) {
                queue.push(message.clone());
            }
        }
    }
}
