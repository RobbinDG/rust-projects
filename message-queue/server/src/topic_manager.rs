use crate::buffer_manager::BufferTypeManager;
use crate::topic::Topic;
use crate::topic_processor::TopicProcessor;

pub type TopicManager = BufferTypeManager<Topic, TopicProcessor>;
