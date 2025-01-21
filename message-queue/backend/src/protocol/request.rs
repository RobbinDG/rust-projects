use crate::protocol::new::message::Message;
use crate::protocol::new::queue_id::QueueId;
use crate::protocol::response::RequestError as ResponseError;
use crate::protocol::status_code::Status;
use crate::protocol::BufferProperties;
use crate::stream_io::StreamIOError;
use serde::{Deserialize, Serialize};
use std::io::Error;
use std::str;
use std::str::Utf8Error;
use crate::protocol::new::routing_error::RoutingError;

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

#[derive(Serialize, Deserialize, Debug)]
pub struct Publish {
    pub message: Message,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Receive {
    pub queue: QueueId,
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

impl Request for Publish {
    type Response = Result<(), RoutingError>;
}

impl Request for Receive {
    type Response = Option<Message>;
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SupportedRequest {
    ListQueues(ListQueues),
    CheckQueue(CheckQueue),
    CreateQueue(CreateQueue),
    DeleteQueue(DeleteQueue),
    GetProperties(GetProperties),
    Publish(Publish),
    Receive(Receive),
}

impl From<ListQueues> for SupportedRequest {
    fn from(value: ListQueues) -> Self {
        SupportedRequest::ListQueues(value)
    }
}

impl From<CheckQueue> for SupportedRequest {
    fn from(value: CheckQueue) -> Self {
        SupportedRequest::CheckQueue(value)
    }
}

impl From<CreateQueue> for SupportedRequest {
    fn from(value: CreateQueue) -> Self {
        SupportedRequest::CreateQueue(value)
    }
}

impl From<DeleteQueue> for SupportedRequest {
    fn from(value: DeleteQueue) -> Self {
        SupportedRequest::DeleteQueue(value)
    }
}

impl From<GetProperties> for SupportedRequest {
    fn from(value: GetProperties) -> Self {
        SupportedRequest::GetProperties(value)
    }
}

impl From<Publish> for SupportedRequest {
    fn from(value: Publish) -> Self {
        SupportedRequest::Publish(value)
    }
}

impl From<Receive> for SupportedRequest {
    fn from(value: Receive) -> Self {
        SupportedRequest::Receive(value)
    }
}

