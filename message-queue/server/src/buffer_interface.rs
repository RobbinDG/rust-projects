use backend::stream_io::StreamIO;
use std::io;
use std::net::TcpStream;
use backend::protocol::BufferAddress;

pub trait BufferInterface<I> {
    fn buffers(&self) -> Vec<(BufferAddress, usize, usize, usize)>;

    fn queue_exists(&self, queue: &I) -> bool;

    fn create(&mut self, name: I);

    fn delete(&mut self, name: &I) -> Option<(Vec<StreamIO>, Vec<StreamIO>)>;

    fn connect_sender(&mut self, queue_name: &I, stream: TcpStream) -> io::Result<()>;

    fn connect_receiver(&mut self, queue_name: &I, stream: TcpStream);

    fn process_queues(&mut self);
}