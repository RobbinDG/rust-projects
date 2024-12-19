use crate::buffer_processor::BufferProcessor;
use crate::topic::Topic;
use backend::protocol::BufferAddress;
use backend::stream_io::StreamIO;
use std::time::Duration;

pub struct TopicProcessor {}

impl TopicProcessor {}

impl BufferProcessor<Topic> for TopicProcessor {
    fn create_buffer(&self) -> Topic {
        Topic::new(Duration::from_secs(50))
    }

    fn address_from_string(&self, string: String) -> BufferAddress {
        BufferAddress::new_topic(string)
    }

    fn process_buffer(
        &mut self,
        senders: &mut Vec<StreamIO>,
        receivers: &mut Vec<StreamIO>,
        topic: &mut Topic,
    ) {
        topic.check_expired();
        for sender in senders {
            while let Ok(message) = sender.read() {
                topic.publish(message);
            }
        }
        for receiver in receivers {
            // We deliberately collect messages to buffer them such that we ensure sending
            // all messages. Using the iterator will update the last_write parameter of the stream
            // after the first write, which is guaranteed to be later than the remaining messages'
            // insert time.
            let messages = topic.unsent_messages(receiver.last_write()).collect::<Vec<_>>();
            for message in messages {
                println!("Writing {:?}", message);
                if let Err(_) = receiver.write(message) {
                    continue;
                }
            }
        }
    }
}
