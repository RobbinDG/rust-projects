use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Eq, Hash, PartialEq)]
pub enum QueueId {
    Queue(String),
    Topic(String),
}
