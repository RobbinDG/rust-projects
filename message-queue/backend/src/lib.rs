pub struct Message {
    payload: String, // TODO byte string?
}

pub struct MessageQueue {
    messages: Vec<Message>,
}

impl MessageQueue {
    pub fn push(&mut self, message: Message) {
        self.messages.push(message)
    }

    pub fn pop(&mut self) -> Option<Message> {
        self.messages.pop()
    }
}


