use crate::request::RequestError;
use serde::{Deserialize, Serialize};
use std::io;
use std::io::{Read, Write};
use std::net::TcpStream;

pub struct StreamIO {
    stream: TcpStream,
}

impl StreamIO {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
        }
    }

    pub fn send_message<T>(&mut self, message: T) -> Result<(), RequestError>
    where
        T: Serialize + for<'a> Deserialize<'a>,
    {
        let payload = postcard::to_allocvec(&message).unwrap();
        self.stream.write_all(&payload)?;
        Ok(())
    }

    pub fn pull_message_from_stream<T>(&mut self) -> Result<T, io::Error>
    where
        T: Serialize + for<'a> Deserialize<'a>,
    {
        let mut buf = [0; 32];
        self.stream.read(&mut buf)?;
        self.stream.flush()?;
        let message: T = postcard::from_bytes(&buf).unwrap();
        Ok(message)
    }
}
