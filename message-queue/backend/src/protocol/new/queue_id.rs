use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum QueueId {
    Queue(String),
    Topic(String),
}
