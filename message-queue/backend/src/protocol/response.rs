use crate::protocol::status_code::Status;
use postcard;
use serde::{Deserialize, Serialize};
use std::str;
use std::str::FromStr;

#[derive(Deserialize, Serialize, Debug)]
pub enum RequestError {
    RequestNotUnderstood,
    ExecFailed(Status),
    CommunicationFailed,
}

impl From<postcard::Error> for RequestError {
    fn from(_: postcard::Error) -> Self {
        RequestError::RequestNotUnderstood
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerResponse
{
    pub payload: String,
}

impl ServerResponse {
    pub fn from_str(message: &str) -> Self {
        ServerResponse {
            payload: String::from_str(message).unwrap(),
        }
    }

    pub fn from_status(status: Status) -> Self {
        Self::from_str(<&str>::from(status))
    }
}

