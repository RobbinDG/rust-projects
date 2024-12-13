use crate::buffer_processor::MessageQueueProcessor;
use crate::message_queue::MessageQueue;
use crate::buffer_manager::BufferManager;

pub type QueueManager = BufferManager<MessageQueue, MessageQueueProcessor>;
