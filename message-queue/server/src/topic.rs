use backend::protocol::Message;
use backend::protocol::{BufferProperties, MessageBuffer};
use std::time::{Duration, SystemTime};
use crate::buffer_channel::ChannelInput;

/// A message that additionally tracks parameters relevant for keeping this message on a
/// topic. To gain access to the underlying message payload, use `send_using()`.
pub struct TopicMessage {
    message: Message,
    inserted_at: SystemTime,
    expire_at: SystemTime,
    times_received: u16,
}

impl TopicMessage {

    /// Sends a message using a desired sending method. The success of the sent messages is used
    /// to track success rate statistics.
    pub fn send_using<R, E, F>(&mut self, f: F) -> Result<R, E>
    where
        F: FnOnce(&Message) -> Result<R, E>,
    {
        f(&self.message).inspect(|_| self.times_received += 1)
    }
}

/// Implements a topic buffer with a fixed time to live. Ordering of messages is not guaranteed.
pub struct Topic {
    properties: BufferProperties,
    ttl: Duration,
    messages: Vec<TopicMessage>,
    pub dlx_channel: ChannelInput,
}

impl Topic {
    pub fn new(properties: BufferProperties, ttl: Duration, dlx_channel: ChannelInput) -> Self {
        Self {
            properties,
            ttl,
            messages: Vec::new(),
            dlx_channel
        }
    }

    pub fn publish(&mut self, message: Message) {
        let now = SystemTime::now();
        self.messages.push(TopicMessage {
            message,
            inserted_at: now,
            expire_at: now + self.ttl,
            times_received: 0,
        })
    }

    /// Checks for expired messages and returns messages that need to be dead-lettered.
    pub fn purge_expired(&mut self) -> Vec<TopicMessage> {
        // TODO this implementation is not efficient whatsoever and reallocates the message
        //  buffer every time.
        let now = SystemTime::now();

        let mut dead = vec![];
        let mut valid = vec![];

        for message in self.messages.drain(..) {
            if message.expire_at <= now {
                valid.push(message);
            } else if message.times_received <= 0 {
                dead.push(message);
            }
        }
        self.messages = valid;
        dead
    }

    pub fn unsent_messages(
        &mut self,
        since: Option<SystemTime>,
    ) -> impl Iterator<Item = &mut TopicMessage> {
        let now = SystemTime::now();
        self.messages.iter_mut().filter(move |m| match since {
            Some(t) => t < m.inserted_at && now < m.expire_at,
            None => true,
        })
    }
}

impl MessageBuffer for Topic {
    fn properties(&self) -> BufferProperties {
        self.properties.clone()
    }

    fn message_count(&self) -> usize {
        self.messages.len()
    }
}
