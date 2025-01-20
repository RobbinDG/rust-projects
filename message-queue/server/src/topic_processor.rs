use crate::buffer_channel::ChannelInput;
use crate::buffer_processor::{BufferInput, BufferProcessor};
use crate::topic::{Topic, TopicMessage};
use backend::protocol::{BufferAddress, BufferProperties};
use backend::stream_io::StreamIO;
use log::error;
use std::time::Duration;

pub struct TopicProcessor {}

impl TopicProcessor {}

impl BufferProcessor<Topic> for TopicProcessor {
    fn create_buffer(&self, properties: BufferProperties, dlx_channel: ChannelInput) -> Topic {
        Topic::new(properties, Duration::from_secs(50), dlx_channel)
    }

    fn address_from_string(&self, string: String) -> BufferAddress {
        BufferAddress::new_topic(string[1..].into())
    }

    fn process_buffer(
        &mut self,
        senders: &mut Vec<BufferInput>,
        receivers: &mut Vec<StreamIO>,
        topic: &mut Topic,
    ) {
        let to_dead_letter = topic.purge_expired();
        for sender in senders {
            while let Some(message) = sender.read() {
                topic.publish(message);
            }
        }
        for receiver in receivers {
            // We deliberately collect messages to buffer them such that we ensure sending
            // all messages. Using the iterator will update the last_write parameter of the stream
            // after the first write, which is guaranteed to be later than the remaining messages'
            // insert time.
            let messages = topic
                .unsent_messages(receiver.last_write())
                .collect::<Vec<_>>();
            for message in messages {
                if let Err(_) = message.send_using(|m| receiver.write_encode(m)) {
                    continue;
                }
            }
        }
        for message in to_dead_letter {
            if let TopicMessage { message: m, .. } = message {
                if let Err(e) = topic.dlx_channel.write(m) {
                    error!("Error when sending message to DLX: {}", e);
                    continue;
                }
            }
        }
    }
}
