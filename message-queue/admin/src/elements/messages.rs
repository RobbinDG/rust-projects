use backend::protocol::queue_id::{QueueId, QueueType};

#[derive(Debug, Clone)]
pub enum UIMessage {
    Refresh,
    NewTableData(Option<Vec<(QueueId, usize, usize, usize)>>),
    NewQueueName(String),
    CreateQueue,
    SelectBufferType(QueueType),
    InspectBuffer(QueueId),
}