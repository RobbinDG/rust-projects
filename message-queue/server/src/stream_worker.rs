use std::net::TcpStream;
use std::time::Duration;
use std::io;
use std::io::{Read, Write};

pub trait StreamWorker {
    fn get_stream(&mut self) -> &mut TcpStream;

    fn init(&mut self, duration: Option<Duration>) -> io::Result<()> {
        self.get_stream().set_read_timeout(duration)?;
        self.get_stream().set_write_timeout(duration)
    }

    fn read(&mut self) -> io::Result<[u8; 32]> {
        let mut buf = [0; 32];
        self.get_stream().read(&mut buf)?;
        self.get_stream().flush()?;
        Ok(buf)
    }
}