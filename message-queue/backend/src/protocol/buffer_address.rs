use serde::{Deserialize, Serialize};

const TOPIC_PREFIX: char = ':';

#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum BufferType {
    Queue,
    Topic,
}

#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct BufferAddress {
    address: String,
    buffer_type: BufferType,
}

impl BufferAddress {
    pub fn new(mut address: String) -> Self {
        address = address
            .replace(" ", "_")
            .replace("\n", "")
            .replace("\r", "")
            .to_lowercase();

        if address.starts_with(TOPIC_PREFIX) {
            Self {
                address: address[1..].into(),
                buffer_type: BufferType::Topic,
            }
        } else {
            Self {
                address,
                buffer_type: BufferType::Queue,
            }
        }
    }

    pub fn to_string(&self) -> String {
        match self.buffer_type {
            BufferType::Queue => self.address.clone(),
            BufferType::Topic => format!("{}{}", TOPIC_PREFIX, self.address),
        }
    }
}