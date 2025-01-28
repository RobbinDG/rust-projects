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

/// A generalised form of [QueueId], used for receiving from queues using
/// generic path arguments.
#[derive(Serialize, Deserialize, Debug, Eq, Hash, PartialEq, Clone)]
pub enum QueueFilter {
    Queue(String),
    Topic(String, TopicLiteral, TopicLiteral),
}

/// A key to uniquely identify a queue implementation, used to send messages one and
/// only one target. To receive using e.g. topic filters, use [QueueFilter].
#[derive(Serialize, Deserialize, Debug, Eq, Hash, PartialEq, Clone)]
pub enum QueueId {
    Queue(String),
    Topic(String, String, String),
}

impl From<QueueId> for QueueFilter {
    fn from(value: QueueId) -> Self {
        match value {
            QueueId::Queue(q) => QueueFilter::Queue(q),
            QueueId::Topic(t, f1, f2) => {
                QueueFilter::Topic(t, TopicLiteral::Name(f1), TopicLiteral::Name(f2))
            }
        }
    }
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
