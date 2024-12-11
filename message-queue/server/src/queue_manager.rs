use std::collections::HashMap;
use backend::message_queue::MessageQueue;
use backend::stream_io::StreamIO;
use std::net::TcpStream;
use backend::message::Message;
use std::io;
use std::io::{Read, Write};

pub struct QueueManager {
    queues: HashMap<String, (Vec<StreamIO>, MessageQueue, Vec<StreamIO>)>,
}

impl QueueManager {
    pub fn new() -> Self {
        QueueManager { queues: HashMap::new() }
    }

    pub fn queues(&self) -> Vec<(String, usize, usize, usize)> {
        // TODO is a Vec a proper return type here?
        self.queues.iter().map(|(k, (i, v, o))| {
            (k.clone(), i.len(), o.len(), v.message_count())
        }).collect()
    }

    pub fn queue_exists(&self, name: &String) -> bool {
        self.queues.contains_key(name)
    }

    pub fn create(&mut self, name: String) {
        self.queues.insert(
            name,
            (Vec::default(), MessageQueue::new_empty(), Vec::default()),
        );
    }

    /// Deletes a queue and all remaining messages. If successful, returns all senders
    /// and receivers on this queue. If the result is not handled, the streams go out of scope
    /// and connections will be closed.
    pub fn delete(&mut self, name: &String) -> Option<(Vec<StreamIO>, Vec<StreamIO>)> {
        println!("Deleting queue {}", name);
        if let Some((senders, _, receivers)) = self.queues.remove(name) {
            return Some((senders, receivers));
        }
        None
    }

    pub fn process_queues(&mut self) {
        println!("Checking queues");
        for (_, (senders, queue, receivers)) in self.queues.iter_mut() {
            for sender in senders {
                match sender.read() {
                    Ok(message) => {
                        println!("{:?}", message);
                        queue.push(message)
                    }
                    Err(_) => {
                        continue;
                    }
                }
            }

            if let Some(recipient) = receivers.get_mut(0) {
                Self::empty_queue_to_stream(queue, recipient);
            }
        }
    }

    fn empty_queue_to_stream(queue: &mut MessageQueue, recipient: &mut StreamIO) {
        while let Some(message) = queue.pop() {
            println!("sending... {:?}", message);
            recipient.write(message).unwrap()
        }
    }

    fn pull_message_from_stream(sender: &mut TcpStream) -> Result<Message, io::Error> {
        let mut buf = [0; 32];
        sender.read(&mut buf)?;
        sender.flush()?;
        let message: Message = postcard::from_bytes(&buf).unwrap();
        Ok(message)
    }

    pub fn connect_sender(&mut self, queue_name: &String, stream: TcpStream) -> io::Result<()> {
        println!("connecting");
        stream.set_nonblocking(true)?;
        if let Some((senders, _, _)) = self.queues.get_mut(queue_name) {
            senders.push(StreamIO::new(stream))
        }
        Ok(())
    }

    pub fn connect_receiver(&mut self, queue_name: &String, stream: TcpStream) {
        if let Some((_, _, recipients)) = self.queues.get_mut(queue_name) {
            recipients.push(StreamIO::new(stream));
        }
    }
}