use backend::protocol::Message;
use std::sync::{Arc, Mutex};

pub struct BufferChannel {
    buffered: Vec<Message>,
    open: bool
}

pub struct ChannelInput {
    channel: Arc<Mutex<BufferChannel>>,
}

impl ChannelInput {
    pub fn write(&mut self, message: Message) -> Result<(), String> {
        let channel = self.channel.lock();
        match channel {
            Ok(c) => {
                if c.open {
                    Ok(c.buffered.push(message))
                } else {
                    Err("Channel was closed.".to_string())
                }
            }
            Err(_) => Err("Communication error.".to_string())
        }
    }

    pub fn close(self) {
        self.channel.lock().unwrap().open = false;
    }
}

pub struct ChannelOutput {
    channel: Arc<Mutex<BufferChannel>>,
}

impl ChannelOutput {
    pub fn read(&mut self) -> Option<Message> {
        self.channel.lock().unwrap().buffered.pop()
    }

    pub fn close(self) {
        self.channel.lock().unwrap().open = false;
    }
}

impl BufferChannel {
    pub fn new() -> (ChannelInput, ChannelOutput) {
        let channel = Arc::new(Mutex::new(BufferChannel {
            buffered: Vec::new(),
            open: false
        }));
        (
            ChannelInput {
                channel: channel.clone(),
            },
            ChannelOutput { channel },
        )
    }
}
