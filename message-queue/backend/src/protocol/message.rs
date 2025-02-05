use crate::protocol::routing_key::RoutingKey;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum TTL {
    Duration(Duration),
    Permanent,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MessagePayload {
    Text(String),
    Json(serde_json::Value),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub payload: MessagePayload,
    pub routing_key: RoutingKey,
    pub ttl: TTL,
}

impl Message {
    pub fn new<P>(payload: P, routing_key: RoutingKey, ttl: TTL) -> Self
    where
        P: Into<MessagePayload>,
    {
        Self {
            payload: payload.into(),
            routing_key,
            ttl,
        }
    }
}

impl Into<MessagePayload> for String {
    fn into(self) -> MessagePayload {
        MessagePayload::Text(self)
    }
}
