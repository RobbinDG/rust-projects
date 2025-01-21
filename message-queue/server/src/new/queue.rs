use backend::protocol::new::message::Message;
use std::time::SystemTime;

struct QueuedMessage {
    message: Message,
    inserted_at: SystemTime,
}

pub struct Queue {
    messages: Vec<QueuedMessage>,
}

impl Queue {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    pub fn push(&mut self, message: Message) {
        self.messages.push(QueuedMessage {
            message,
            inserted_at: SystemTime::now(),
        });
    }

    pub fn pop(&mut self) -> Option<Message> {
        match self.messages.pop() {
            Some(QueuedMessage { message, .. }) => Some(message),
            _ => None,
        }
    }
}
