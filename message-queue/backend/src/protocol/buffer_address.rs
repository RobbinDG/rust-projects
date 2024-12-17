use serde::{Deserialize, Serialize};

const TOPIC_PREFIX: char = ':';

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum BufferType {
    Queue,
    Topic,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct BufferAddress {
    address: String,
    buffer_type: BufferType,
}

impl BufferAddress {
    pub fn new(mut address: String) -> Self {
        address = Self::sanitise_name(address);

        if address.starts_with(TOPIC_PREFIX) {
            Self::new_queue(address[1..].into())
        } else {
            Self::new_topic(address)
        }
    }

    pub fn new_queue(address: String) -> Self {
        Self {
            address: Self::sanitise_name(address),
            buffer_type: BufferType::Queue,
        }
    }

    pub fn new_topic(address: String) -> Self {
        Self {
            address: Self::sanitise_name(address),
            buffer_type: BufferType::Topic,
        }
    }

    fn sanitise_name(address: String) -> String {
        address
            .replace(" ", "_")
            .replace("\n", "")
            .replace("\r", "")
            .to_lowercase()
    }

    pub fn to_string(&self) -> String {
        println!("{self:?}");
        match self.buffer_type {
            BufferType::Queue => self.address.clone(),
            BufferType::Topic => format!("{}{}", TOPIC_PREFIX, self.address),
        }
    }

    pub fn buffer_type(&self) -> BufferType {
        self.buffer_type
    }
}
