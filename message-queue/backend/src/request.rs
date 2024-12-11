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

#[derive(Debug, Serialize, Deserialize)]
pub enum AdminRequest {
    ListQueues(ListQueues),
    CheckQueue(CheckQueue),
    CreateQueue(CreateQueue),
}

impl From<ListQueues> for AdminRequest {
    fn from(value: ListQueues) -> Self {
        AdminRequest::ListQueues(value)
    }
}
impl From<CheckQueue> for AdminRequest {
    fn from(value: CheckQueue) -> Self {
        AdminRequest::CheckQueue(value)
    }
}
impl From<CreateQueue> for AdminRequest {
    fn from(value: CreateQueue) -> Self {
        AdminRequest::CreateQueue(value)
    }
}

impl RequestType for AdminRequest {
    type Response = Vec<u8>;
}
