use crate::protocol::status_code::Status;
use postcard;
use serde::{Deserialize, Serialize};
use std::str;
use std::str::FromStr;
use std::sync::PoisonError;

#[derive(Deserialize, Serialize, Debug)]
pub enum ResponseError {
    RequestNotUnderstood,
    ExecFailed,
    CommunicationFailed,
    PoisonError,
}

impl<T> From<PoisonError<T>> for ResponseError {
    fn from(_: PoisonError<T>) -> Self {
        ResponseError::PoisonError
    }
}

impl From<postcard::Error> for ResponseError {
    fn from(_: postcard::Error) -> Self {
        ResponseError::RequestNotUnderstood
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

