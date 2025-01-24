use backend::protocol::QueueProperties;
use std::collections::HashMap;
use backend::protocol::client_id::ClientID;
use backend::protocol::message::Message;
use log::{debug, info};
use crate::queue::{DequeuedMessage, Queue};

pub struct MessageTopic {
    properties: QueueProperties,
    client_queues: HashMap<ClientID, Queue>,
}

impl MessageTopic {
    pub fn new(properties: QueueProperties) -> Self {
        Self {
            properties,
            client_queues: HashMap::new(),
        }
    }

    pub fn publish(&mut self, message: Message) {
        for queue in self.client_queues.values_mut() {
            queue.push(message.clone());
        }
    }

    pub fn receive(&mut self, client: &ClientID) -> Option<DequeuedMessage> {
        debug!("Receiving message for {:?}", client);
        self.client_queues
            .get_mut(client)
            .and_then(|queue| queue.pop())
    }

    pub fn register_client(&mut self, client: ClientID) {
        info!("Creating topic buffer for {:?}", client);
        self.client_queues.insert(client.clone(), Queue::new());
    }

    pub fn deregister_client(&mut self, client: &ClientID) {
        self.client_queues.remove(client);
    }

    pub fn properties(&self) -> &QueueProperties {
        &self.properties
    }
}