use crate::queue::MessageState;
use crate::queue_store::{Publisher, QueueStore};
use backend::protocol::client_id::ClientID;
use backend::protocol::message::{Message, TTL};
use backend::protocol::queue_id::{QueueFilter, QueueId};
use backend::protocol::routing_error::RoutingError;
use backend::protocol::routing_key::{DLXPreference, RoutingKey};
use backend::protocol::{QueueProperties, SystemQueueProperties, UserQueueProperties};
use log::{debug, error, info, warn};
use std::sync::{Arc, LockResult, Mutex};

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
            default_dlx.clone().into(),
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
    /// type (Queue/Topic). If sending fails, the message is sent to its requested DLX.
    ///
    /// # Arguments
    ///
    /// * `message`: the message to publish, including the routing key according to which it
    ///     will be routed.
    ///
    /// returns: `Result<(), RoutingError>` a potential routing error if it occurs
    ///     during publishing.
    pub fn publish(&mut self, message: Message) -> Result<(), RoutingError> {
        let publish_err = {
            let mut binding = self.queues.lock()?;
            match binding.publisher(&message.routing_key.id.clone()) {
                None => Err((RoutingError::NotFound, message)),
                Some(mut publisher) => {
                    info!("Publishing to {:?}", &message.routing_key.id);
                    match publisher.publish(message) {
                        Ok(_) => Ok(()),
                        Err(msg) => Err((RoutingError::NoRecipients, msg)),
                    }
                }
            }
        };

        if let Err((err, msg)) = publish_err {
            self.send_to_dlx(msg)?;
            match &err {
                RoutingError::NoRecipients => {},  // This is not a reason to Err the requester.
                _ => return Err(err),
            }
        }

        Ok(())
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
    pub fn receive_valid(&mut self, queue: &QueueFilter, for_client: ClientID) -> Option<Message> {
        let (message, to_dlx) = self.receive_until_valid(queue, for_client);
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

    fn receive_until_valid(
        &mut self,
        queue: &QueueFilter,
        for_client: ClientID,
    ) -> (Option<Message>, Vec<Message>) {
        // TODO the starting capacity can be chosen intelligently if we track i.e. the shortest
        //  ttl of all messages currently in the queue.
        let mut dlx_messages = vec![];
        match self.queues.lock() {
            Ok(mut binding) => match binding.receiver(&for_client, queue) {
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
                let queue_dlx = match self.queues.lock()?.properties(&id.to_top_level()) {
                    None => self.default_dlx.clone(),
                    Some(properties) => properties
                        .user
                        .dlx
                        .clone()
                        .unwrap_or(self.default_dlx.clone()),
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
