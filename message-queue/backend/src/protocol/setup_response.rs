use serde::{Deserialize, Serialize};
use crate::protocol::BufferAddress;

#[derive(Serialize, Deserialize, Debug)]
pub enum SetupResponse {
    Disconnect,
    Admin,
    Sender(BufferAddress),
    Receiver(BufferAddress),
}