use crate::protocol::new::queue_id::QueueId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum SetupResponse {
    Disconnect,
    Admin,
    Sender,
    Receiver(QueueId),
}