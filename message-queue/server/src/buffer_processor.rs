use crate::message_queue::{MessageBuffer, MessageQueue};
use backend::stream_io::StreamIO;

pub trait BufferProcessor<T>
where
    T: MessageBuffer,
{
    fn create_buffer(&self) -> T;
    fn process_buffer(
        &mut self,
        senders: &mut Vec<StreamIO>,
        receivers: &mut Vec<StreamIO>,
        queue: &mut T,
    );
}

pub struct MessageQueueProcessor {}

impl BufferProcessor<MessageQueue> for MessageQueueProcessor {
    fn create_buffer(&self) -> MessageQueue {
        MessageQueue::new_empty()
    }

    fn process_buffer(
        &mut self,
        senders: &mut Vec<StreamIO>,
        receivers: &mut Vec<StreamIO>,
        queue: &mut MessageQueue,
    ) {
        for sender in senders {
            match sender.read() {
                Ok(message) => {
                    println!("{:?}", message);
                    queue.push(message)
                }
                Err(_) => {
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
            println!("sending... {:?}", message);
            recipient.write(message).unwrap()
        }
    }
}
