use crate::buffer_type_manager::BufferTypeManager;
use crate::buffer_processor::MessageQueueProcessor;
use crate::message_queue::MessageQueue;

pub type QueueManager = BufferTypeManager<MessageQueue, MessageQueueProcessor>;
