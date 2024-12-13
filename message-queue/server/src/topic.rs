use crate::message_buffer::MessageBuffer;
use backend::protocol::Message;

pub struct Topic {}

impl Topic {
    fn push(&mut self, message: Message) {
        todo!()
    }

    fn pop(&mut self) -> Option<Message> {
        todo!()
    }
}

impl MessageBuffer for Topic {
    fn message_count(&self) -> usize {
        todo!()
    }
}
