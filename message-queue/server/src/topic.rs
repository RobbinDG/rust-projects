use crate::message_buffer::MessageBuffer;
use backend::protocol::Message;
use std::time::{Duration, SystemTime};

struct TopicMessage {
    message: Message,
    inserted_at: SystemTime,
    expire_at: SystemTime,
}

/// Implements a topic buffer with a fixed time to live. Ordering of messages is not guaranteed.
pub struct Topic {
    ttl: Duration,
    messages: Vec<TopicMessage>,
}

impl Topic {
    pub fn new(ttl: Duration) -> Self {
        Self {
            ttl,
            messages: Vec::new(),
        }
    }

    pub fn publish(&mut self, message: Message) {
        let now = SystemTime::now();
        self.messages.push(TopicMessage {
            message,
            inserted_at: now,
            expire_at: now + self.ttl,
        })
    }

    pub fn check_expired(&mut self) {
        let now = SystemTime::now();
        self.messages.retain(|message| message.expire_at > now);
        // TODO dead letter removed messages
    }

    pub fn unsent_messages(&mut self, since: Option<SystemTime>) -> impl Iterator<Item = &Message> {
        self.check_expired();  // TODO this might be called too frequently and lead to inefficiency
        self.messages
            .iter()
            .filter(move |m| match since {
                Some(t) => t < m.inserted_at,
                None => true,
            })
            .map(|m| &m.message)
    }
}

impl MessageBuffer for Topic {
    fn message_count(&self) -> usize {
        self.messages.len()
    }
}
