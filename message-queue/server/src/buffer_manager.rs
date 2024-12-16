use std::collections::HashMap;
use backend::stream_io::StreamIO;
use std::net::TcpStream;
use std::io;
use backend::protocol::BufferAddress;
use crate::buffer_processor::BufferProcessor;
use crate::message_buffer::MessageBuffer;

pub struct BufferManager<T, P>
where
    T: MessageBuffer,
    P: BufferProcessor<T>,
{
    buffer_processor: P,
    queues: HashMap<BufferAddress, (Vec<StreamIO>, T, Vec<StreamIO>)>,
}

impl<T, P> BufferManager<T, P>
where
    T: MessageBuffer,
    P: BufferProcessor<T>,
{
    pub fn new(buffer_processor: P) -> Self {
        BufferManager {
            buffer_processor,
            queues: HashMap::new(),
        }
    }

    pub fn queues(&self) -> Vec<(String, usize, usize, usize)> {
        // TODO is a Vec a proper return type here?
        self.queues
            .iter()
            .map(|(k, (i, v, o))| (k.to_string(), i.len(), o.len(), v.message_count()))
            .collect()
    }

    pub fn queue_exists(&self, queue: &BufferAddress) -> bool {
        self.queues.contains_key(queue)
    }

    pub fn create(&mut self, name: BufferAddress) {
        self.queues.insert(
            name,
            (
                Vec::default(),
                self.buffer_processor.create_buffer(),
                Vec::default(),
            ),
        );
    }

    /// Deletes a queue and all remaining messages. If successful, returns all senders
    /// and receivers on this queue. If the result is not handled, the streams go out of scope
    /// and connections will be closed.
    pub fn delete(&mut self, name: &BufferAddress) -> Option<(Vec<StreamIO>, Vec<StreamIO>)> {
        println!("Deleting queue {:?}", name);
        if let Some((senders, _, receivers)) = self.queues.remove(name) {
            return Some((senders, receivers));
        }
        None
    }

    pub fn process_queues(&mut self) {
        println!("Checking queues");
        for (_, (senders, queue, receivers)) in self.queues.iter_mut() {
            self.buffer_processor
                .process_buffer(senders, receivers, queue);
        }
    }

    pub fn connect_sender(&mut self, queue_name: &BufferAddress, stream: TcpStream) -> io::Result<()> {
        stream.set_nonblocking(true)?;
        if let Some((senders, _, _)) = self.queues.get_mut(queue_name) {
            senders.push(StreamIO::new(stream))
        }
        Ok(())
    }

    pub fn connect_receiver(&mut self, queue_name: &BufferAddress, stream: TcpStream) {
        if let Some((_, _, recipients)) = self.queues.get_mut(queue_name) {
            recipients.push(StreamIO::new(stream));
        }
    }
}