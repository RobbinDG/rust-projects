use serde::{Deserialize, Serialize};
use std::io::Error;
use std::str;
use std::str::Utf8Error;

#[derive(Debug)]
pub enum RequestError {
    IO(Error),
    Parsing(Utf8Error),
    Internal(String),
}

impl From<Error> for RequestError {
    fn from(value: Error) -> Self {
        RequestError::IO(value)
    }
}

impl From<Utf8Error> for RequestError {
    fn from(value: Utf8Error) -> Self {
        RequestError::Parsing(value)
    }
}

impl From<String> for RequestError {
    fn from(value: String) -> Self {
        RequestError::Internal(value)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerRequest {
    ListQueues,
    CheckQueue(String),
    CreateQueue(String),
    PutMessage(String, String),
}