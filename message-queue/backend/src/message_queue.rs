use crate::message::Message;

pub struct MessageQueue {
    messages: Vec<Message>,
}

impl MessageQueue {
    pub fn new_empty() -> MessageQueue {
        MessageQueue { messages: Vec::default() }
    }

    pub fn put(&mut self, message: Message) {
        self.messages.push(message);
    }
}

impl MessageQueue {
    pub fn push(&mut self, message: Message) {
        self.messages.push(message)
    }

    pub fn pop(&mut self) -> Option<Message> {
        self.messages.pop()
    }
}