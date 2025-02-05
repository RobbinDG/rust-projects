use std::cmp::max;
use crate::queue::{DequeuedMessage, Queue};
use crate::topic_filter_tree::TopicFilterTree;
use backend::protocol::client_id::ClientID;
use backend::protocol::message::Message;
use backend::protocol::queue_id::TopicLiteral;
use backend::protocol::QueueProperties;
use log::{debug, info};
use std::collections::{HashMap, HashSet};
use backend::protocol::routing_error::RoutingError;

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

    pub fn create_subtopic(&mut self, filter: (String, Option<String>)) {
        self.clients_by_filter.create_subtopic(filter.0.clone());
        if let Some(f) = filter.1 {
            self.clients_by_filter.create_subsubtopic(filter.0, f)
        }
    }

    pub fn subtopic_exists(&self, filter: (String, String)) -> bool {
        // TODO this is really inefficient (but quick)
        self.get_subtopics().get(&filter.0).and_then(|sub| sub.get(&filter.1)).is_some()
    }

    pub fn is_filter_valid(&self, filter: (&TopicLiteral, &TopicLiteral)) -> bool {
       self.clients_by_filter.is_filter_nonempty(filter)
    }

    pub fn receive(&mut self, client: &ClientID) -> Option<DequeuedMessage> {
        debug!("Receiving message for {:?}", client);
        self.client_queues
            .get_mut(client)?
            .pop()
    }

    pub fn register_client(
        &mut self,
        client: ClientID,
        topic_filter: (TopicLiteral, TopicLiteral),
    ) {
        info!("Creating topic buffer for {:?}", client);
        self.clients_by_filter.insert(client.clone(), &vec![topic_filter.0, topic_filter.1]);
        self.client_queues.insert(client, Queue::new());
    }

    pub fn deregister_client(&mut self, client: &ClientID) {
        self.clients_by_filter.remove(client);
        self.client_queues.remove(client);
    }

    pub fn properties(&self) -> &QueueProperties {
        &self.properties
    }

    pub fn publish(&mut self, message: Message, f1: String, f2: String) -> Result<(), Message> {
        let clients = self.clients_by_filter.get_clients((f1, f2));

        if clients.len() <= 0 {
            return Err(message);
        }

        for client in clients {
            if let Some(queue) = self.client_queues.get_mut(client) {
                queue.push(message.clone());
            }
        }
        Ok(())
    }

    pub fn message_count(&self) -> usize {
        let mut max_count = 0usize;
        for queue in self.client_queues.values() {
            max_count = max_count.max(queue.len());
        }
        max_count
    }
}
