use crate::queue_store::QueueStore;
use backend::protocol::client_id::ClientID;
use backend::protocol::queue_id::{QueueFilter, TopLevelQueueId};
use log::info;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Maintains the active subscriptions to queues, and forwards resource allocation
/// to support receive requests to only the subscribed queue.
pub struct SubscriptionManager {
    queue_store: Arc<Mutex<QueueStore>>,
    subscriptions: HashMap<ClientID, QueueFilter>,
}

impl SubscriptionManager {
    pub fn new(queue_store: Arc<Mutex<QueueStore>>) -> Self {
        Self {
            queue_store,
            subscriptions: HashMap::new(),
        }
    }

    /// Subscribes a connection to a queue, ensuring that subsequent message requests from
    /// that queue are possible after this call.
    ///
    /// # Arguments
    ///
    /// * `client`: the client to subscribe.
    /// * `queue_id`: the queue to subscribe the client to.
    ///
    /// returns: `bool` if the subscription was correctly made.
    pub fn subscribe(&mut self, client: ClientID, queue_id: QueueFilter) -> bool {
        let mut queues = match self.queue_store.lock() {
            Ok(binding) => {
                if !binding.is_filter_valid(&queue_id) {
                    self.subscriptions.remove(&client);
                    return false;
                }
                binding
            }
            Err(_) => return false,
        };

        info!("Subscribing {:?} to queue {:?}", client, queue_id);

        self.subscriptions
            .entry(client.clone())
            .and_modify(|existing| {
                queues.deregister_client(existing, &client);
                *existing = queue_id.clone();
                queues.register_client(&queue_id, client.clone());
            })
            .or_insert_with(|| {
                queues.register_client(&queue_id, client);
                queue_id
            });
        true
    }

    /// Checks if a client is subscribed to a provided queue to avoid hitting a queue
    /// store with unnecessary request.
    ///
    /// # Arguments
    ///
    /// * `client`: the client to check the subscription for.
    /// * `queue_id`: the queue the client is asked if it is subscribed to.
    ///
    /// returns: `bool` whether the client is subscribed or not.
    pub fn subscribed(&self, client: &ClientID, queue_id: &QueueFilter) -> bool {
        self.subscriptions
            .get(client)
            .map_or(false, |subscription| subscription == queue_id)
    }

    /// Retrieves the current subscription of a client, if any.
    ///
    /// # Arguments
    ///
    /// * `client`: the client to retrieve the subscription for.
    ///
    /// returns: `Option<&QueueId>` the queue ID of the current subscription of the client.
    pub fn subscription(&self, client: &ClientID) -> Option<&QueueFilter> {
        self.subscriptions.get(client)
    }

    pub fn subscriber_counts(&self) -> HashMap<TopLevelQueueId, usize> {
        let mut counts = HashMap::new();
        for (_, filter) in &self.subscriptions {
            counts
                .entry(filter.to_top_level())
                .and_modify(|v| *v += 1)
                .or_insert(1usize);
        }
        counts
    }
}
