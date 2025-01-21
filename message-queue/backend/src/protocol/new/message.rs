use crate::protocol::new::routing_key::RoutingKey;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    payload: String,
    routing_key: RoutingKey,
    ttl: Duration,
}