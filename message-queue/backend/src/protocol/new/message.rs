use crate::protocol::new::routing_key::RoutingKey;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub payload: String,
    pub routing_key: RoutingKey,
    pub ttl: Duration,
}

impl Message {
    pub fn new(payload: String, routing_key: RoutingKey, ttl: Duration) -> Self {
        Self {
            payload,
            routing_key,
            ttl,
        }
    }
}
