use serde::{Deserialize, Serialize};
use crate::request::{RequestType, SetModeResponse};

#[derive(Debug, Serialize, Deserialize)]
pub enum SetupRequest {
    Admin,
    Sender(String),
    Receiver(String),
}

impl RequestType for SetupRequest {
    type Response = SetModeResponse;
}