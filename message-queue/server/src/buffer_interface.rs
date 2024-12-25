use backend::protocol::{BufferAddress, BufferProperties};
use backend::stream_io::StreamIO;
use std::io;

pub trait BufferInterface<I> {
    fn buffers(&self) -> Vec<(BufferAddress, usize, usize, usize)>;

    fn queue_exists(&self, queue: &I) -> bool;

    fn buffer_properties(&self, buffer: &I) -> Option<BufferProperties>;

    fn create(&mut self, name: I, properties: BufferProperties);

    fn delete(&mut self, name: &I) -> Option<(Vec<StreamIO>, Vec<StreamIO>)>;

    fn connect_sender(&mut self, queue_name: &I, stream: StreamIO) -> io::Result<()>;

    fn connect_receiver(&mut self, queue_name: &I, stream: StreamIO);

    fn process_queues(&mut self);
}