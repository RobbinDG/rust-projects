use backend::protocol::{BufferAddress, BufferType};

#[derive(Debug, Clone)]
pub enum UIMessage {
    Refresh,
    NewQueueName(String),
    CreateQueue,
    DeleteQueue(BufferAddress),
    SelectBufferType(BufferType)
}