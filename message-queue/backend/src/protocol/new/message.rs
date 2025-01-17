use std::time::Duration;
use crate::protocol::new::routing_key::RoutingKey;

pub struct Message {
    payload: String,
    routing_key: RoutingKey,
    ttl: Duration,
}