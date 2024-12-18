use crate::buffer_processor::BufferProcessor;
use crate::topic::Topic;
use backend::protocol::{BufferAddress, Message};
use backend::stream_io::StreamIO;

pub struct TopicProcessor {}

impl TopicProcessor {}

impl BufferProcessor<Topic> for TopicProcessor {
    fn create_buffer(&self) -> Topic {
        Topic {}
    }

    fn address_from_string(&self, string: String) -> BufferAddress {
        BufferAddress::new_topic(string)
    }

    fn process_buffer(
        &mut self,
        senders: &mut Vec<StreamIO>,
        receivers: &mut Vec<StreamIO>,
        _: &mut Topic,
    ) {
        let mut messages: Vec<Message> = vec![];
        for sender in senders {
            while let Ok(message) = sender.read() {
                messages.push(message);
            }
        }
        for receiver in receivers {
            for message in &messages {
                if let Err(_) = receiver.write(&message.clone()) {
                    continue;
                }
            }
        }
    }
}
