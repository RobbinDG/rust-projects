use crate::new::queue::MessageState;
use crate::new::queue_store::QueueStore;
use backend::protocol::new::message::{Message, TTL};
use backend::protocol::new::queue_id::QueueId;
use backend::protocol::new::routing_error::RoutingError;
use backend::protocol::new::routing_key::{DLXPreference, RoutingKey};
use log::{debug, error, warn};
use std::sync::{Arc, Mutex};
use std::time::Duration;

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
        /// Publish message to its intended destination queue.
        self.queues
            .lock()?
            .publisher(&message.routing_key.id)
            .ok_or(RoutingError::NotFound)
            .map(|mut p| p.publish(message))
    }

    pub fn receive_valid(&mut self, queue_id: &QueueId) -> Option<Message> {
        let (message, to_dlx) = self.receive_until_valid(queue_id);
        for m in to_dlx {
            if let Err(err) = self.send_to_dlx(m) {
                match err {
                    RoutingError::DropOnDLX => {
                        warn!("Message dropped due to DLX rule.")
                    }
                    e => {
                        error!("Uncaught error when sending message to DLX: {:?}", e)
                    }
                }
            }
        }
        message
    }

    fn receive_until_valid(&mut self, queue_id: &QueueId) -> (Option<Message>, Vec<Message>) {
        // TODO the starting capacity can be chosen intelligently if we track i.e. the shortest
        //  ttl of all messages currently in the queue.
        let mut dlx_messages = vec![];
        match self.queues.lock() {
            Ok(mut binding) => match binding.receiver(queue_id) {
                Some(mut receiver) => {
                    while let Some(message) = receiver.receive() {
                        if let MessageState::Dead = message.state {
                            dlx_messages.push(message.message);
                        } else {
                            return (Some(message.message), dlx_messages);
                        }
                    }
                    (None, dlx_messages)
                }
                None => (None, dlx_messages),
            },
            Err(_) => (None, dlx_messages),
        }
    }

    fn send_to_dlx(&mut self, mut message: Message) -> Result<(), RoutingError> {
        debug!("Sending message to DLX {:?}", message.routing_key.dlx);

        /// Deconstruct message into its components.
        let Message {
            payload,
            routing_key,
            ..
        } = message;
        let RoutingKey { dlx, .. } = routing_key;

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
        let new_message = Message::new(payload, new_routing_key, TTL::Permanent);

        self.publish(new_message)
    }
}
