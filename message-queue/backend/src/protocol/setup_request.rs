use serde::{Deserialize, Serialize};
use crate::protocol::request::RequestType;
use crate::protocol::SetupResponse;

#[derive(Debug, Serialize, Deserialize)]
pub enum SetupRequest {
    Admin,
    Sender(String),
    Receiver(String),
}

impl RequestType for SetupRequest {
    type Response = SetupResponse;
}