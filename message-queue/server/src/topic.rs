use crate::message_buffer::MessageBuffer;

pub struct Topic {}

impl Topic {}

impl MessageBuffer for Topic {
    fn message_count(&self) -> usize {
        0
    }
}
