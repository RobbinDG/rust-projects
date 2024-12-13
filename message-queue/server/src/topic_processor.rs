use backend::stream_io::StreamIO;
use crate::buffer_processor::BufferProcessor;
use crate::topic::Topic;

pub struct TopicProcessor {}

impl TopicProcessor {}

impl BufferProcessor<Topic> for TopicProcessor {
    fn create_buffer(&self) -> Topic {
        Topic {}
    }

    fn process_buffer(&mut self, senders: &mut Vec<StreamIO>, receivers: &mut Vec<StreamIO>, queue: &mut Topic) {
        todo!()
    }
}