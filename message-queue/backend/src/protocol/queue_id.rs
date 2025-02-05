use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

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

impl Display for TopicLiteral {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// The top level definition of a [QueueId], without any subtopics.
#[derive(Serialize, Deserialize, Debug, Eq, Hash, PartialEq, Clone)]
pub enum TopLevelQueueId {
    Queue(String),
    Topic(String),
}

impl TopLevelQueueId {
    pub fn to_string(&self) -> String {
        match self {
            TopLevelQueueId::Queue(name) => name.clone(),
            TopLevelQueueId::Topic(name) => format!(
                "{}{}{}{}{}",
                name.to_string(),
                TOPIC_DELIMITER,
                TopicLiteral::Wildcard.to_string(),
                TOPIC_DELIMITER,
                TopicLiteral::Wildcard.to_string(),
            ),
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

impl QueueFilter {
    pub fn to_string(&self) -> String {
        match self {
            QueueFilter::Queue(name) => name.clone(),
            QueueFilter::Topic(a, b, c) => format!(
                "{}{}{}{}{}",
                a.to_string(),
                TOPIC_DELIMITER,
                b.to_string(),
                TOPIC_DELIMITER,
                c.to_string()
            ),
        }
    }

    pub fn to_top_level(&self) -> TopLevelQueueId {
        match &self {
            QueueFilter::Queue(q) => TopLevelQueueId::Queue(q.clone()),
            QueueFilter::Topic(t, _, _) => TopLevelQueueId::Topic(t.clone()),
        }
    }
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

    pub fn to_top_level(&self) -> TopLevelQueueId {
        match self {
            QueueId::Queue(q) => TopLevelQueueId::Queue(q.clone()),
            QueueId::Topic(t, _, _) => TopLevelQueueId::Topic(t.clone()),
        }
    }
}

impl From<QueueId> for TopLevelQueueId {
    fn from(value: QueueId) -> Self {
        value.to_top_level()
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, Hash, PartialEq, Clone)]
pub enum NewQueueId {
    Queue(String),
    Topic(String, Option<(String, Option<String>)>),
}

impl From<QueueId> for NewQueueId {
    fn from(value: QueueId) -> Self {
        match value {
            QueueId::Queue(name) => NewQueueId::Queue(name),
            QueueId::Topic(name, f1, f2) => NewQueueId::Topic(name, Some((f1, Some(f2)))),
        }
    }
}
