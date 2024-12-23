use crate::protocol::ResponseError;
use serde::{Deserialize, Serialize};
use std::io;
use std::io::{ErrorKind, Read, Write};
use std::net::{Shutdown, TcpStream};
use std::time::{Duration, SystemTime};

const BUFFER_SIZE: usize = 1024;

pub struct StreamIO {
    stream: TcpStream,
    last_read: Option<SystemTime>,
    last_write: Option<SystemTime>,
}

#[derive(Debug)]
pub enum StreamIOError {
    /// For errors related to the transfer of packets.
    Stream(io::Error),
    /// For errors with serialisation of messages.
    Codec(postcard::Error),
}

impl From<StreamIOError> for String {
    fn from(value: StreamIOError) -> Self {
        match value {
            StreamIOError::Stream(e) => e.to_string(),
            StreamIOError::Codec(e) => e.to_string(),
        }
    }
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
/// or incorrectly formatted. Connections will automatically be shutdown once the stream
/// goes out of scope.
impl StreamIO {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            last_read: None,
            last_write: None,
        }
    }

    pub fn set_timeout(&mut self, duration: Option<Duration>) -> io::Result<()> {
        self.stream.set_read_timeout(duration)?;
        self.stream.set_write_timeout(duration)
    }

    pub fn set_nonblocking(&mut self, nonblocking: bool) -> io::Result<()> {
        self.stream.set_nonblocking(nonblocking)
    }

    /// Write a struct to the stream, after first encoding it. The struct must
    /// be serialisable and deserialisable by `serde`.
    pub fn write<T>(&mut self, message: &T) -> Result<(), StreamIOError>
    where
        T: Serialize + for<'a> Deserialize<'a>,
    {
        let result = Ok(self.stream.write_all(&postcard::to_allocvec(message)?)?);
        self.last_write = Some(SystemTime::now());
        result
    }

    /// Read a struct from the stream, after first decoding it. The struct must
    /// be serialisable and deserialisable by `serde`.
    pub fn read<T>(&mut self) -> Result<T, StreamIOError>
    where
        T: Serialize + for<'a> Deserialize<'a>,
    {
        let mut buf = [0; BUFFER_SIZE];
        self.stream.read(&mut buf)?;
        self.stream.flush()?;
        let result = Ok(postcard::from_bytes(&buf)?);
        self.last_read = Some(SystemTime::now());
        result
    }

    /// Read a `Result` containing the desired struct as `Ok` and a `crate::stream_io::StreamIOError` as `Err`
    /// from the stream. This assumes that the `Ok` value is encoded prior to being wrapped
    /// in the `Result`, and therefore is doubly encoded. The struct must be serialisable
    /// and deserialisable by `serde`.
    pub fn read_encoded_result<T>(&mut self) -> Result<Result<T, ResponseError>, StreamIOError>
    where
        T: Serialize + for<'a> Deserialize<'a>,
    {
        let response: Result<Vec<u8>, ResponseError> = self.read()?;
        Ok(match response {
            Ok(r) => Ok(postcard::from_bytes(r.as_slice())?),
            Err(err) => Err(err),
        })
    }

    pub fn last_read(&self) -> Option<SystemTime> {
        self.last_read
    }

    pub fn last_write(&self) -> Option<SystemTime> {
        self.last_write
    }

    pub fn reset(&mut self) {
        self.last_write = None;
        self.last_read = None;
    }
}

impl Drop for StreamIO {
    fn drop(&mut self) {
        if let Err(e) = self.stream.shutdown(Shutdown::Both) {
            match e.kind() {
                ErrorKind::BrokenPipe
                | ErrorKind::ConnectionAborted
                | ErrorKind::ConnectionRefused
                | ErrorKind::ConnectionReset
                | ErrorKind::NotConnected => return,
                _ => Err(e).unwrap(),
            }
        }
    }
}
