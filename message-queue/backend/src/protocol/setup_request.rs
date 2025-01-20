use crate::protocol::request::Request;
use crate::protocol::{BufferAddress, SetupResponse};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum SetupRequest {
    Admin,
    Sender(BufferAddress),
    Receiver(BufferAddress),
}

impl Request for SetupRequest {
    type Response = SetupResponse;
}
