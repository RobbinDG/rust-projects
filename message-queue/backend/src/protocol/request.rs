use crate::protocol::message::Message;
use crate::protocol::queue_id::{NewQueueId, QueueFilter, QueueId, TopLevelQueueId};
use crate::protocol::queue_properties::UserQueueProperties;
use crate::protocol::routing_error::RoutingError;
use crate::protocol::status_code::Status;
use crate::protocol::QueueProperties;
use serde::{Deserialize, Serialize};
use std::str;

pub trait Request: Serialize + for<'de> Deserialize<'de> + Send {
    type Response: Serialize + for<'de> Deserialize<'de> + Sized + Send;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListQueues {}

#[derive(Serialize, Deserialize, Debug)]
pub struct CheckQueue {
    pub queue_address: QueueId,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateQueue {
    pub queue_address: NewQueueId,
    pub properties: UserQueueProperties,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteQueue {
    pub queue_name: TopLevelQueueId,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetProperties {
    pub queue: TopLevelQueueId,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Publish {
    pub message: Message,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Subscribe {
    pub queue: QueueFilter,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Receive {}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetTopicBreakdown {
    pub topic_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetSubscription {}

impl Request for ListQueues {
    type Response = Vec<(TopLevelQueueId, usize, usize)>;
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

impl Request for GetTopicBreakdown {
    type Response = Option<Vec<(String, Vec<String>)>>;
}

impl Request for GetSubscription {
    type Response = Option<QueueFilter>;
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
    GetTopicBreakdown(GetTopicBreakdown),
    GetSubscription(GetSubscription),
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

impl From<GetTopicBreakdown> for SupportedRequest {
    fn from(value: GetTopicBreakdown) -> Self {
        SupportedRequest::GetTopicBreakdown(value)
    }
}

impl From<GetSubscription> for SupportedRequest {
    fn from(value: GetSubscription) -> Self {
        SupportedRequest::GetSubscription(value)
    }
}
