#[derive(Debug, Clone)]
pub enum UIMessage {
    Refresh,
    NewQueueName(String),
    CreateQueue,
    DeleteQueue(String),
}