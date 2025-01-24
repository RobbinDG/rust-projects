use serde::{Deserialize, Serialize};

const TOPIC_DELIMITER: &str = ":";

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
pub enum TopicLiteral {
    Name(String),
    Wildcard,
}

impl TopicLiteral {
    pub fn to_string(&self) -> String {
        match self {
            TopicLiteral::Name(name) => name.clone(),
            TopicLiteral::Wildcard => "*".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, Hash, PartialEq, Clone)]
pub enum QueueId {
    Queue(String),
    Topic(String, TopicLiteral, TopicLiteral),
}

impl QueueId {
    pub fn queue_type(&self) -> QueueType {
        match self {
            QueueId::Queue(_) => QueueType::Queue,
            QueueId::Topic(_, _, _) => QueueType::Topic,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            QueueId::Queue(name) => name.clone(),
            QueueId::Topic(a, b, c) => format!(
                "{}{}{}{}{}",
                a.to_string(),
                TOPIC_DELIMITER,
                b.to_string(),
                TOPIC_DELIMITER,
                c.to_string()
            ),
        }
    }
}
