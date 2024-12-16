use crate::protocol::request::RequestType;
use crate::protocol::{BufferAddress, SetupResponse};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum SetupRequest {
    Admin,
    Sender(BufferAddress),
    Receiver(BufferAddress),
}

impl RequestType for SetupRequest {
    type Response = SetupResponse;
}
