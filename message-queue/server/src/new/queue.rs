use std::time::SystemTime;
use backend::protocol::new::message::Message;

struct QueuedMessage {
    message: Message,
    inserted_at: SystemTime,
}

pub struct Queue {
    messages: Vec<QueuedMessage>,
}

impl Queue {
    pub fn new() -> Self {
        todo!()
    }

    pub fn push(&mut self, message: Message) {
        todo!()
    }

    pub fn pop(&mut self) -> Option<Message> {
        todo!()
    }

}