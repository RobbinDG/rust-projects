use crate::buffer_interface::BufferInterface;
use crate::buffer_processor::{BufferInput, BufferProcessor};
use backend::protocol::{BufferProperties, Message, MessageBuffer};
use backend::protocol::BufferAddress;
use backend::stream_io::StreamIO;
use log::warn;
use std::collections::HashMap;
use std::io;
use crate::buffer_channel::{BufferChannel, ChannelInput};

pub struct BufferTypeManager<T, P>
where
    T: MessageBuffer,
    P: BufferProcessor<T>,
{
    buffer_processor: P,
    queues: HashMap<String, (Vec<BufferInput>, T, Vec<StreamIO>)>,
}

impl<T, P> BufferTypeManager<T, P>
where
    T: MessageBuffer,
    P: BufferProcessor<T>,
{
    pub fn new(buffer_processor: P) -> Self {
        Self {
            buffer_processor,
            queues: HashMap::new(),
        }
    }

    pub fn dead_letter_channel(&mut self, buffer_name: &String) -> ChannelInput {
        let (i, o) = BufferChannel::new();
        if let Some((senders, _, _)) = self.queues.get_mut(buffer_name) {
            senders.push(BufferInput::Buffer(o));
        }
        i
    }

    pub fn create(&mut self, name: String, properties: BufferProperties, dlx_channel: ChannelInput) {
        self.queues.insert(
            name,
            (
                Vec::default(),
                self.buffer_processor.create_buffer(properties, dlx_channel),
                Vec::default(),
            ),
        );
    }
}

impl<T, P> BufferInterface<String> for BufferTypeManager<T, P>
where
    T: MessageBuffer,
    P: BufferProcessor<T>,
{
    fn buffers(&self) -> Vec<(BufferAddress, usize, usize, usize)> {
        self.queues
            .iter()
            .map(|(k, (i, v, o))| (self.buffer_processor.address_from_string(k.clone()), i.len(), o.len(), v.message_count()))
            .collect()
    }

    fn queue_exists(&self, queue: &String) -> bool {
        self.queues.contains_key(queue)
    }

    fn buffer_properties(&self, buffer: &String) -> Option<BufferProperties> {
        self.queues.get(buffer).map(|(_, t, _)| t.properties())
    }

    /// Deletes a queue and all remaining messages. If successful, returns all senders
    /// and receivers on this queue. If the result is not handled, the streams go out of scope
    /// and connections will be closed.
    fn delete(&mut self, name: &String) -> Option<(Vec<BufferInput>, Vec<StreamIO>)> {
        if let Some((_, buf, _)) = self.queues.get(name) {
            if buf.properties().system_buffer {
                return None; // TODO a "refused" response would be appropriate in this case.
            }
        }
        warn!("Deleting buffer {:?}", name);
        if let Some((senders, _, receivers)) = self.queues.remove(name) {
            return Some((senders, receivers));
        }
        None
    }

    fn connect_sender(&mut self, queue_name: &String, mut stream: StreamIO) -> io::Result<()> {
        if let Some((senders, _, _)) = self.queues.get_mut(queue_name) {
            senders.push(BufferInput::Stream(stream))
        }
        Ok(())
    }

    fn connect_receiver(&mut self, queue_name: &String, stream: StreamIO) {
        if let Some((_, _, recipients)) = self.queues.get_mut(queue_name) {
            recipients.push(stream);
        }
    }

    fn process_queues(&mut self) {
        for (_, (senders, queue, receivers)) in self.queues.iter_mut() {
            self.buffer_processor
                .process_buffer(senders, receivers, queue);
        }
    }
}
