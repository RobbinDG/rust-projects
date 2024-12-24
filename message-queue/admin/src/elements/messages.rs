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
    ConnectionUpdated(ConnectionInterfaceMessage),
}

impl From<ConnectionInterfaceMessage> for UIMessage {
    fn from(value: ConnectionInterfaceMessage) -> Self {
        UIMessage::ConnectionUpdated(value)
    }
}