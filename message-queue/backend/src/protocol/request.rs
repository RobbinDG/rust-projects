use crate::protocol::message::Message;
use crate::protocol::queue_id::QueueId;
use crate::protocol::queue_properties::UserQueueProperties;
use crate::protocol::routing_error::RoutingError;
use crate::protocol::status_code::Status;
use crate::protocol::QueueProperties;
use serde::{Deserialize, Serialize};
use std::str;

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
    pub properties: UserQueueProperties,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteQueue {
    pub queue_name: QueueId,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetProperties {
    pub queue: QueueId,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Publish {
    pub message: Message,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Subscribe {
    pub queue: QueueId,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Receive {}

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
    type Response = Option<QueueProperties>;
}

impl Request for Publish {
    type Response = Result<(), RoutingError>;
}

impl Request for Subscribe {
    type Response = Status;
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
    Subscribe(Subscribe),
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

impl From<Subscribe> for SupportedRequest {
    fn from(value: Subscribe) -> Self {
        SupportedRequest::Subscribe(value)
    }
}

impl From<Receive> for SupportedRequest {
    fn from(value: Receive) -> Self {
        SupportedRequest::Receive(value)
    }
}
