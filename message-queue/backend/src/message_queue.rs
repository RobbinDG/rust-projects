use std::io::Write;
use crate::message::Message;
use std::net::TcpStream;
use postcard::to_allocvec;

pub struct MessageQueue {
    messages: Vec<Message>,
    pub recipients: Vec<TcpStream>,
}

impl MessageQueue {
    pub fn new_empty() -> MessageQueue {
        MessageQueue { messages: Vec::default(), recipients: Vec::default() }
    }

    pub fn put(&mut self, message: Message) {
        self.messages.push(message);
    }
}

impl MessageQueue {
    pub fn push(&mut self, message: Message) {
        if let Some(mut recipient) = self.recipients.get(0) {
            let payload = to_allocvec(&message).unwrap();
            recipient.write_all(&payload).unwrap();
        }
        // self.messages.push(message)
    }

    pub fn pop(&mut self) -> Option<Message> {
        self.messages.pop()
    }
}