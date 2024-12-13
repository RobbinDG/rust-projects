use crate::buffer_manager::BufferManager;
use crate::topic::Topic;
use crate::topic_processor::TopicProcessor;

pub type TopicManager = BufferManager<Topic, TopicProcessor>;
