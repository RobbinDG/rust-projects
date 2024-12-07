use crate::status_code::Status;
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

impl From<postcard::Error> for RequestError {
    fn from(value: postcard::Error) -> Self {
        RequestError::Internal(value.to_string())
    }
}

impl RequestError {
    pub fn to_string(&self) -> String {
        match self {
            RequestError::IO(e) => format!("IO error: {}", e.to_string()),
            RequestError::Parsing(e) => format!("IO error: {}", e.to_string()),
            RequestError::Internal(e) => format!("IO error: {}", e.to_string()),
        }
    }
}

pub trait RequestType {
    type Response: Serialize + for<'de> Deserialize<'de> + Sized;
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SetModeResponse {
    Disconnect,
    Admin,
    Sender(String),
    Receiver(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListQueues {}

#[derive(Serialize, Deserialize, Debug)]
pub struct CheckQueue {
    pub queue_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateQueue {
    pub queue_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MakeAdmin {}

#[derive(Serialize, Deserialize, Debug)]
pub struct MakeSender {
    pub destination_queue: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MakeReceiver {
    pub origin_queue: String,
}

pub enum ServerResponse {
    QueueList(String),
    Success,
}

impl RequestType for ListQueues {
    type Response = Vec<(String, usize, usize, usize)>;
}

impl RequestType for CheckQueue {
    type Response = Status;
}

impl RequestType for CreateQueue {
    type Response = Status;
}

impl RequestType for MakeAdmin {
    type Response = SetModeResponse;
}

impl RequestType for MakeSender {
    type Response = SetModeResponse;
}

impl RequestType for MakeReceiver {
    type Response = SetModeResponse;
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerRequest {
    ListQueues(ListQueues),
    CheckQueue(CheckQueue),
    CreateQueue(CreateQueue),
}
