use backend::protocol::Message;

pub trait MessageBuffer {
    fn push(&mut self, message: Message);
    fn pop(&mut self) -> Option<Message>;
    fn message_count(&self) -> usize;
}

pub struct MessageQueue {
    messages: Vec<Message>,
}

impl MessageQueue {
    pub fn new_empty() -> MessageQueue {
        MessageQueue {
            messages: Vec::default(),
        }
    }
}

impl MessageBuffer for MessageQueue {
    fn push(&mut self, message: Message) {
        self.messages.push(message);
    }

    fn pop(&mut self) -> Option<Message> {
        self.messages.pop()
    }

    fn message_count(&self) -> usize {
        self.messages.len()
    }
}
