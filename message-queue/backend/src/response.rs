use crate::status_code::Status;
use serde::{Deserialize, Serialize};
use std::str;
use std::str::FromStr;

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

