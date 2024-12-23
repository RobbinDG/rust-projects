use crate::protocol::buffer_address::BufferAddress;
use crate::protocol::status_code::Status;
use crate::stream_io::StreamIOError;
use serde::{Deserialize, Serialize};
use std::io::Error;
use std::str;
use std::str::Utf8Error;
use crate::protocol::response::ResponseError;

#[derive(Debug)]
pub enum RequestError {
    IO(StreamIOError),
    Parsing(Utf8Error),
    Internal(ResponseError),
}

impl From<Error> for RequestError {
    fn from(value: Error) -> Self {
        RequestError::from(StreamIOError::Stream(value))
    }
}

impl From<StreamIOError> for RequestError {
    fn from(value: StreamIOError) -> Self {
        RequestError::IO(value)
    }
}

impl From<Utf8Error> for RequestError {
    fn from(value: Utf8Error) -> Self {
        RequestError::Parsing(value)
    }
}

impl From<ResponseError> for RequestError {
    fn from(value: ResponseError) -> Self {
        RequestError::Internal(value)
    }
}

pub trait RequestType {
    type Response: Serialize + for<'de> Deserialize<'de> + Sized;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListQueues {}

#[derive(Serialize, Deserialize, Debug)]
pub struct CheckQueue {
    pub queue_address: BufferAddress,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateQueue {
    pub queue_address: BufferAddress,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteQueue {
    pub queue_name: BufferAddress,
}

impl RequestType for ListQueues {
    type Response = Vec<(BufferAddress, usize, usize, usize)>;
}

impl RequestType for CheckQueue {
    type Response = Status;
}

impl RequestType for CreateQueue {
    type Response = Status;
}

impl RequestType for DeleteQueue {
    type Response = Status;
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AdminRequest {
    ListQueues(ListQueues),
    CheckQueue(CheckQueue),
    CreateQueue(CreateQueue),
    DeleteQueue(DeleteQueue),
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

impl From<DeleteQueue> for AdminRequest {
    fn from(value: DeleteQueue) -> Self {
        AdminRequest::DeleteQueue(value)
    }
}

impl RequestType for AdminRequest {
    type Response = Vec<u8>;
}
