use backend::protocol::new::message::{Message, TTL};
use std::time::SystemTime;

struct QueuedMessage {
    message: Message,
    inserted_at: SystemTime,
}

pub enum MessageState {
    Valid,
    Dead,
}

pub struct DequeuedMessage {
    pub message: Message,
    pub state: MessageState,
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

    pub fn pop(&mut self) -> Option<DequeuedMessage> {
        match self.messages.pop() {
            Some(QueuedMessage {
                message,
                inserted_at,
            }) => {
                let valid = match message.ttl {
                    TTL::Duration(d) => SystemTime::now() < inserted_at + d,
                    TTL::Permanent => true,
                } ;
                Some(DequeuedMessage {
                    message,
                    state: if valid {
                        MessageState::Valid
                    } else {
                        MessageState::Dead
                    },
                })
            }
            _ => None,
        }
    }
}
