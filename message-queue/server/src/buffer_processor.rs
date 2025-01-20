use crate::buffer_channel::{ChannelInput, ChannelOutput};
use crate::message_queue::MessageQueue;
use backend::protocol::MessageBuffer;
use backend::protocol::{BufferAddress, BufferProperties, Message};
use backend::stream_io::{StreamIO, StreamIOError};
use log::{debug, error};

pub enum BufferInput {
    Stream(StreamIO),
    Buffer(ChannelOutput),
}

impl BufferInput {
    pub fn read(&mut self) -> Option<Message> {
        match self {
            BufferInput::Stream(stream) => stream
                .read()
                .inspect_err(|err| {
                    error!("Error reading from stream input: {:?}", err);
                })
                .ok(),
            BufferInput::Buffer(buf) => buf.read(),
        }
    }
}

pub trait BufferProcessor<T>
where
    T: MessageBuffer,
{
    fn create_buffer(&self, properties: BufferProperties, dlx_channel: ChannelInput) -> T;

    fn address_from_string(&self, string: String) -> BufferAddress;
    fn process_buffer(
        &mut self,
        senders: &mut Vec<BufferInput>,
        receivers: &mut Vec<StreamIO>,
        queue: &mut T,
    );
}

pub struct MessageQueueProcessor {}

impl BufferProcessor<MessageQueue> for MessageQueueProcessor {
    fn create_buffer(&self, properties: BufferProperties, dlx_channel: ChannelInput) -> MessageQueue {
        MessageQueue::new_empty(properties)
    }

    fn address_from_string(&self, string: String) -> BufferAddress {
        BufferAddress::new_queue(string)
    }

    fn process_buffer(
        &mut self,
        senders: &mut Vec<BufferInput>,
        receivers: &mut Vec<StreamIO>,
        queue: &mut MessageQueue,
    ) {
        for sender in senders {
            match sender.read() {
                Some(message) => {
                    debug!("{:?}", message);
                    queue.push(message)
                }
                None => {
                    continue;
                }
            }
        }

        if let Some(recipient) = receivers.get_mut(0) {
            Self::empty_queue_to_stream(queue, recipient);
        }
    }
}

impl MessageQueueProcessor {
    fn empty_queue_to_stream(queue: &mut MessageQueue, recipient: &mut StreamIO) {
        while let Some(message) = queue.pop() {
            debug!("sending... {:?}", message);
            if let Err(e) = recipient.write_encode(&message) {
                error!("{:?}", e);
            }
        }
    }
}
