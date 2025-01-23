use crate::queue::MessageState;
use crate::queue_store::QueueStore;
use backend::protocol::message::{Message, TTL};
use backend::protocol::queue_id::QueueId;
use backend::protocol::routing_error::RoutingError;
use backend::protocol::routing_key::{DLXPreference, RoutingKey};
use backend::protocol::{QueueProperties, SystemQueueProperties, UserQueueProperties};
use log::{debug, error, warn};
use std::sync::{Arc, Mutex};

const DEFAULT_DLX_NAME: &'static str = "default_dlx";

/// A struct responsible for sending messages to the correct destination queue given its
/// routing key. By extension, the Router will also handle sending messages to
/// dead-letter exchanges (DLX) and updating their routing keys when conditions change.
/// Additionally, the router will define and create the default DLX.
pub struct Router {
    queues: Arc<Mutex<QueueStore>>,
    default_dlx: QueueId,
}

impl Router {
    pub fn new(queues: Arc<Mutex<QueueStore>>) -> Self {
        let default_dlx = QueueId::Queue(DEFAULT_DLX_NAME.into());
        queues.lock().unwrap().create(
            default_dlx.clone(),
            QueueProperties {
                system: SystemQueueProperties { is_system: true },
                user: UserQueueProperties {
                    is_dlx: true,
                    dlx: None,
                },
            },
        );
        Self {
            queues,
            default_dlx,
        }
    }

    /// Publish a message to its intended destination queue, regardless of queue implementation
    /// type (Queue/Topic).
    ///
    /// # Arguments
    ///
    /// * `message`: the message to publish, including the routing key according to which it
    ///     will be routed.
    ///
    /// returns: `Result<(), RoutingError>` a potential routing error if it occurs
    ///     during publishing.
    pub fn publish(&mut self, message: Message) -> Result<(), RoutingError> {
        self.queues
            .lock()?
            .publisher(&message.routing_key.id)
            .ok_or(RoutingError::NotFound)
            .map(|mut p| p.publish(message))
    }

    /// Receive a message that is valid at the time of calling. During this call, invalid
    /// messages received at the head of the requested queue will be sent to their
    /// set DLX preference.
    ///
    /// # Arguments
    ///
    /// * `queue_id`: the queue of which to request a message.
    ///
    /// returns: `Option<Message>` the received message, if there is one.
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

    fn send_to_dlx(&mut self, message: Message) -> Result<(), RoutingError> {
        debug!("Sending message to DLX {:?}", message.routing_key.dlx);

        // Deconstruct message into its components.
        let Message {
            payload,
            routing_key,
            ..
        } = message;
        let RoutingKey { dlx, id } = routing_key;

        // Derive the new routing key from the DLX preference.
        let new_routing_key = match dlx {
            DLXPreference::Default => {
                RoutingKey::new(self.default_dlx.clone(), DLXPreference::Drop)
            }
            DLXPreference::Queue => {
                let queue_dlx = match self.queues.lock()?.properties(&id) {
                    None => self.default_dlx.clone(),
                    Some(properties) => properties.user.dlx.clone().unwrap_or(self.default_dlx.clone()),
                };
                RoutingKey::new(queue_dlx, DLXPreference::Drop)
            }
            DLXPreference::Override(dlx) => RoutingKey::new(dlx, DLXPreference::Drop),
            DLXPreference::Drop => {
                return Err(RoutingError::DropOnDLX);
            }
        };

        // Construct the new message with updated routing key.
        let new_message = Message::new(payload, new_routing_key, TTL::Permanent);

        self.publish(new_message)
    }
}
