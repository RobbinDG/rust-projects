use backend::protocol::{BufferProperties, MessageBuffer};
use backend::protocol::Message;

pub struct MessageQueue {
    properties: BufferProperties,
    messages: Vec<Message>,
}

impl MessageQueue {
    pub fn new_empty(properties: BufferProperties) -> MessageQueue {
        MessageQueue {
            properties,
            messages: Vec::default(),
        }
    }

    pub fn push(&mut self, message: Message) {
        self.messages.push(message);
    }

    pub fn pop(&mut self) -> Option<Message> {
        self.messages.pop()
    }
}

impl MessageBuffer for MessageQueue {
    fn properties(&self) -> BufferProperties {
        self.properties.clone()
    }

    fn message_count(&self) -> usize {
        self.messages.len()
    }
}
