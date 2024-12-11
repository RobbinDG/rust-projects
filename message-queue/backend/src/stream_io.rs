use serde::{Deserialize, Serialize};
use std::io;
use std::io::{Read, Write};
use std::net::TcpStream;

pub struct StreamIO {
    stream: TcpStream,
}

#[derive(Debug)]
pub enum StreamIOError {
    /// For errors related to the transfer of packets.
    Stream(io::Error),
    /// For errors with serialisation of messages.
    Codec(postcard::Error),
    /// For errors during the processing of the request.
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

/// A wrapper around `std::net::TcpStream` that provides helper methods for strongly
/// typed encoded messages and exposes error handling for when packets are not received
/// or incorrectly formatted.
impl StreamIO {
    pub fn new(stream: TcpStream) -> Self {
        Self { stream }
    }

    /// Write a struct to the stream, after first encoding it. The struct must
    /// be serialisable and deserialisable by `serde`.
    pub fn write<T>(&mut self, message: T) -> Result<(), StreamIOError>
    where
        T: Serialize + for<'a> Deserialize<'a>,
    {
        Ok(self.stream.write_all(&postcard::to_allocvec(&message)?)?)
    }

    /// Read a struct from the stream, after first decoding it. The struct must
    /// be serialisable and deserialisable by `serde`.
    pub fn read<T>(&mut self) -> Result<T, StreamIOError>
    where
        T: Serialize + for<'a> Deserialize<'a>,
    {
        let mut buf = [0; 32];
        self.stream.read(&mut buf)?;
        self.stream.flush()?;
        Ok(postcard::from_bytes(&buf)?)
    }

    /// Read a `Result` containing the desired struct as `Ok` and a `crate::stream_io::StreamIOError` as `Err`
    /// from the stream. This assumes that the `Ok` value is encoded prior to being wrapped
    /// in the `Result`, and therefore is doubly encoded. The struct must be serialisable
    /// and deserialisable by `serde`.
    pub fn read_encoded_result<T>(&mut self) -> Result<T, StreamIOError>
    where
        T: Serialize + for<'a> Deserialize<'a>,
    {
        let response: Result<Vec<u8>, String> = self.read()?;
        match response {
            Ok(r) => Ok(postcard::from_bytes(r.as_slice())?),
            Err(e) => Err(StreamIOError::Request(e)),
        }
    }
}
