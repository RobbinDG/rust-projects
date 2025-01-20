use crate::protocol::buffer_address::BufferAddress;
use crate::protocol::response::RequestError as ResponseError;
use crate::protocol::status_code::Status;
use crate::stream_io::StreamIOError;
use serde::{Deserialize, Serialize};
use std::io::Error;
use std::str;
use std::str::Utf8Error;
use crate::protocol::BufferProperties;
use crate::protocol::new::queue_id::QueueId;

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

pub trait Request {
    type Response: Serialize + for<'de> Deserialize<'de> + Sized;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListQueues {}

#[derive(Serialize, Deserialize, Debug)]
pub struct CheckQueue {
    pub queue_address: QueueId,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateQueue {
    pub queue_address: QueueId,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteQueue {
    pub queue_name: QueueId,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetProperties {
    pub buffer: QueueId,
}

impl Request for ListQueues {
    type Response = Vec<(QueueId, usize, usize, usize)>;
}

impl Request for CheckQueue {
    type Response = Status;
}

impl Request for CreateQueue {
    type Response = Status;
}

impl Request for DeleteQueue {
    type Response = Status;
}

impl Request for GetProperties {
    type Response = BufferProperties;
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AdminRequest {
    ListQueues(ListQueues),
    CheckQueue(CheckQueue),
    CreateQueue(CreateQueue),
    DeleteQueue(DeleteQueue),
    GetProperties(GetProperties),
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

impl From<GetProperties> for AdminRequest {
    fn from(value: GetProperties) -> Self {
        AdminRequest::GetProperties(value)
    }
}

impl Request for AdminRequest {
    type Response = Vec<u8>;
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SupportedRequest {
    ListQueues(ListQueues),
    CheckQueue(CheckQueue),
    CreateQueue(CreateQueue),
    DeleteQueue(DeleteQueue),
    GetProperties(GetProperties),
}

impl From<CreateQueue> for SupportedRequest {
    fn from(r: CreateQueue) -> Self {
        SupportedRequest::CreateQueue(r)
    }
}
