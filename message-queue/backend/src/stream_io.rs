use std::fmt::Debug;
use crate::protocol::codec::{decode, encode, CodecError};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use crate::protocol::client_id::ClientID;
use crate::protocol::request_error::RequestError;

const BUFFER_SIZE: usize = 1024;

pub struct StreamIO {
    stream: TcpStream,
    last_read: Option<SystemTime>,
    last_write: Option<SystemTime>,
}

#[derive(Debug)]
pub enum StreamIOError {
    /// For errors related to the transfer of packets.
    Stream(std::io::Error),
    /// For errors with serialisation of messages.
    Codec(postcard::Error),
}

impl From<CodecError> for StreamIOError {
    fn from(e: CodecError) -> Self {
        let CodecError(pce) = e;
        StreamIOError::Codec(pce)
    }
}

impl From<StreamIOError> for String {
    fn from(value: StreamIOError) -> Self {
        match value {
            StreamIOError::Stream(e) => e.to_string(),
            StreamIOError::Codec(e) => e.to_string(),
        }
    }
}


impl From<std::io::Error> for StreamIOError {
    fn from(value: std::io::Error) -> Self {
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

    pub async fn write(&mut self, data: &Vec<u8>) -> Result<(), StreamIOError> {

        let result = Ok(self.stream.write_all(data).await?);
        self.last_write = Some(SystemTime::now());
        result
    }

    /// Write a struct to the stream, after first encoding it. The struct must
    /// be serialisable and deserialisable by `serde`.
    pub async fn write_encode<T>(&mut self, message: &T) -> Result<(), StreamIOError>
    where
        T: Serialize + for<'a> Deserialize<'a> + Debug,
    {
        let encoded = encode(message)?;
        self.write(&encoded).await
    }

    /// Read a struct from the stream, after first decoding it. The struct must
    /// be serialisable and deserialisable by `serde`.
    pub async fn read<T>(&mut self) -> Result<T, StreamIOError>
    where
        T: Serialize + for<'a> Deserialize<'a>,
    {
        let mut buf = [0; BUFFER_SIZE];
        self.stream.read(&mut buf).await?;
        let result = Ok(decode(&buf.to_vec())?);
        self.last_read = Some(SystemTime::now());
        result
    }

    /// Try to read a struct from the stream, after first decoding it. The struct must
    /// be serialisable and deserialisable by `serde`.
    pub async fn try_read<T>(&mut self) -> Result<T, StreamIOError>
    where
        T: Serialize + for<'a> Deserialize<'a>,
    {
        let mut buf = [0; BUFFER_SIZE];
        self.stream.try_read(&mut buf)?;
        let result = Ok(postcard::from_bytes(&buf)?);
        self.last_read = Some(SystemTime::now());
        result
    }

    /// Read a `Result` containing the desired struct as `Ok` and a `crate::stream_io::StreamIOError` as `Err`
    /// from the stream. This assumes that the `Ok` value is encoded prior to being wrapped
    /// in the `Result`, and therefore is doubly encoded. The struct must be serialisable
    /// and deserialisable by `serde`.
    pub async fn read_encoded_result<T>(
        &mut self,
    ) -> Result<Result<T, RequestError>, StreamIOError>
    where
        T: Serialize + for<'a> Deserialize<'a>,
    {
        let response: Result<Vec<u8>, RequestError> = self.read().await?;
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

    pub fn client_id(&self) -> tokio::io::Result<ClientID> {
        Ok(ClientID::TcpSocket(self.stream.peer_addr()?))
    }
}
