use crate::protocol::new::routing_key::RoutingKey;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum TTL {
    Duration(Duration),
    Permanent,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub payload: String,
    pub routing_key: RoutingKey,
    pub ttl: TTL,
}

impl Message {
    pub fn new(payload: String, routing_key: RoutingKey, ttl: TTL) -> Self {
        Self {
            payload,
            routing_key,
            ttl,
        }
    }
}
