use serde::{Deserialize, Serialize};
use std::io;
use std::io::{Read, Write};
use std::net::TcpStream;

pub struct StreamIO {
    stream: TcpStream,
}

#[derive(Debug)]
pub enum StreamIOError {
    Stream(io::Error),
    Codec(postcard::Error),
    Request(String),
}

impl From<io::Error> for StreamIOError {
    fn from(value: io::Error) -> Self {
        StreamIOError::Stream(value)
    }
}

impl From<postcard::Error> for StreamIOError {
    fn from(err: postcard::Error) -> Self {
        StreamIOError::Codec(err)
    }
}

impl StreamIO {
    pub fn new(stream: TcpStream) -> Self {
        Self { stream }
    }

    pub fn send_message<T>(&mut self, message: T) -> Result<(), StreamIOError>
    where
        T: Serialize + for<'a> Deserialize<'a>,
    {
        Ok(self.stream.write_all(&postcard::to_allocvec(&message)?)?)
    }

    pub fn pull_message_from_stream<T>(&mut self) -> Result<T, StreamIOError>
    where
        T: Serialize + for<'a> Deserialize<'a>,
    {
        let mut buf = [0; 32];
        self.stream.read(&mut buf)?;
        self.stream.flush()?;
        Ok(postcard::from_bytes(&buf)?)
    }

    pub fn pull_admin_response<T>(&mut self) -> Result<T, StreamIOError>
    where
        T: Serialize + for<'a> Deserialize<'a>,
    {
        let response: Result<Vec<u8>, String> = self.pull_message_from_stream()?;
        match response {
            Ok(r) => Ok(postcard::from_bytes(r.as_slice())?),
            Err(e) => Err(StreamIOError::Request(e)),
        }
    }
}
