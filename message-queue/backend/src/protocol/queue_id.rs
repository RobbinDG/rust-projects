use serde::{Deserialize, Serialize};

const TOPIC_PREFIX: &str = ":";

#[derive(Serialize, Deserialize, Debug, Eq, Hash, PartialEq, Clone, Copy)]
pub enum QueueType {
    Queue,
    Topic,
}

impl QueueType {
    pub fn to_str(&self) -> &str {
        match self {
            QueueType::Queue => "Queue",
            QueueType::Topic => "Topic",
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, Hash, PartialEq, Clone)]
pub enum QueueId {
    Queue(String),
    Topic(String),
}

impl QueueId {
    pub fn new(name: String, type_: QueueType) -> Self {
        match type_ {
            QueueType::Queue => QueueId::Queue(name),
            QueueType::Topic => QueueId::Topic(name),
        }
    }

    pub fn queue_type(&self) -> QueueType {
        match self {
            QueueId::Queue(_) => QueueType::Queue,
            QueueId::Topic(_) => QueueType::Topic,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            QueueId::Queue(name) => name.clone(),
            QueueId::Topic(name) => format!("{}{}", TOPIC_PREFIX, name),
        }
    }
}
