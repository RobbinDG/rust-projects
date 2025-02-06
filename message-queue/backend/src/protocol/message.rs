use crate::protocol::codec::{decode, encode, CodecError};
use crate::protocol::routing_key::RoutingKey;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum TTL {
    Duration(Duration),
    Permanent,
}

pub enum PayloadDecodeError {
    Codec(CodecError),
    NotBlob,
}

impl From<CodecError> for PayloadDecodeError {
    fn from(value: CodecError) -> Self {
        Self::Codec(value)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MessagePayload {
    Text(String),
    Blob(Vec<u8>),
}

impl MessagePayload {
    pub fn encode_blob<T>(data: &T) -> Result<Self, CodecError>
    where
        T: Serialize + for<'a> Deserialize<'a>,
    {
        Ok(Self::Blob(encode(data)?))
    }

    pub fn decode_blob<T>(&self) -> Result<T, PayloadDecodeError>
    where
        T: Serialize + for<'a> Deserialize<'a>,
    {
        match self {
            MessagePayload::Blob(data) => Ok(decode::<T>(data)?),
            _ => Err(PayloadDecodeError::NotBlob),
        }
    }
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
