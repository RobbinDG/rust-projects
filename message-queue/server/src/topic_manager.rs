use std::collections::HashMap;
use std::io;
use std::net::TcpStream;
use backend::stream_io::StreamIO;

pub struct TopicManager {
    topics: HashMap<String, (Vec<StreamIO>, Vec<StreamIO>)>,
}

impl TopicManager {
    pub fn new() -> Self {
        Self {
            topics: HashMap::new(),
        }
    }

    pub fn process_topics(&mut self) {

    }

    pub fn connect_sender(&mut self, queue_name: &String, stream: TcpStream) -> io::Result<()> {
        stream.set_nonblocking(true)?;
        if let Some((senders, _)) = self.topics.get_mut(queue_name) {
            senders.push(StreamIO::new(stream))
        }
        Ok(())
    }

    pub fn connect_receiver(&mut self, queue_name: &String, stream: TcpStream) {
        if let Some((_, recipients)) = self.topics.get_mut(queue_name) {
            recipients.push(StreamIO::new(stream));
        }
    }
}