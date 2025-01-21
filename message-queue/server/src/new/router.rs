use crate::new::queue_store::QueueStore;
use backend::protocol::new::message::Message;
use backend::protocol::new::queue_id::QueueId;
use backend::protocol::new::routing_error::RoutingError;
use backend::protocol::new::routing_key::{DLXPreference, RoutingKey};
use std::sync::{Arc, Mutex};

pub struct Router {
    queues: Arc<Mutex<QueueStore>>,
    default_dlx: QueueId,
}

impl Router {
    pub fn new(queues: Arc<Mutex<QueueStore>>) -> Self {
        let default_dlx = QueueId::Queue("default_dlx".into());
        queues.lock().unwrap().create(default_dlx.clone());
        Self {
            queues,
            default_dlx,
        }
    }

    pub fn publish(&mut self, message: Message) -> Result<(), RoutingError> {
        /// Deconstruct message into its components.
        let Message {
            payload,
            routing_key,
            ttl,
        } = message;
        let RoutingKey { id, dlx } = routing_key;

        /// Derive the new routing key from the DLX preference.
        let new_routing_key = match dlx {
            DLXPreference::Default => {
                RoutingKey::new(self.default_dlx.clone(), DLXPreference::Drop)
            }
            DLXPreference::Queue => {
                todo!()
            }
            DLXPreference::Override(dlx) => RoutingKey::new(dlx, DLXPreference::Drop),
            DLXPreference::Drop => {
                return Err(RoutingError::DropOnDLX);
            }
        };

        /// Construct the new message with updated routing key.
        let new_message = Message::new(payload, new_routing_key, ttl);

        /// Publish message to its intended destination queue.
        self.queues
            .lock()?
            .publisher(&id)
            .ok_or(RoutingError::NotFound)
            .map(|mut p| p.publish(new_message))
    }
}
