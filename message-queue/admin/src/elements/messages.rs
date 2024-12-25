use backend::protocol::{BufferAddress, BufferType};
use crate::elements::connection_interface::ConnectionInterfaceMessage;

#[derive(Debug, Clone)]
pub enum UIMessage {
    Refresh,
    NewQueueName(String),
    CreateQueue,
    DeleteQueue(BufferAddress),
    SelectBufferType(BufferType),
    InspectBuffer(BufferAddress),
}