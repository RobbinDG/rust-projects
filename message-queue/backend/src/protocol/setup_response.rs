use serde::{Deserialize, Serialize};
use crate::protocol::BufferAddress;
use crate::protocol::new::queue_id::QueueId;

#[derive(Serialize, Deserialize, Debug)]
pub enum SetupResponse {
    Disconnect,
    Admin,
    Sender,
    Receiver(QueueId),
}