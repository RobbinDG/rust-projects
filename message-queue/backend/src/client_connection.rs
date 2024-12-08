use std::fmt::Debug;
use crate::message::Message;
use crate::request::{RequestType, ServerRequest};
use crate::stream_io::{StreamIO, StreamIOError};
use serde::{Deserialize, Serialize};
use std::io;
use std::net::{TcpStream, ToSocketAddrs};

pub struct ConnectionConfig<T>
where
    T: ToSocketAddrs + Clone,
{
    address: T,
}
pub struct DisconnectedClient<T>
where
    T: ToSocketAddrs + Clone,
{
    config: ConnectionConfig<T>,
}
pub struct ConnectedClient<T>
where
    T: ToSocketAddrs + Clone,
{
    config: ConnectionConfig<T>,
    stream: StreamIO,
}

pub struct ConnectionError<T>
where
    T: ToSocketAddrs + Clone,
{
    pub error_body: io::Error,
    pub server: DisconnectedClient<T>,
}

impl<T: ToSocketAddrs + Clone> DisconnectedClient<T> {
    pub fn new(addr: T) -> DisconnectedClient<T> {
        DisconnectedClient {
            config: ConnectionConfig { address: addr },
        }
    }

    pub fn connect(self) -> Result<ConnectedClient<T>, ConnectionError<T>> {
        match TcpStream::connect(&self.config.address) {
            Ok(stream) => Ok(ConnectedClient {
                config: self.config,
                stream: StreamIO::new(stream),
            }),
            Err(e) => Err(ConnectionError {
                error_body: e,
                server: self,
            }),
        }
    }
}

impl<T: ToSocketAddrs + Clone> ConnectedClient<T> {
    pub fn transfer_request<R>(&mut self, request: R) -> Result<R::Response, StreamIOError>
    where
        R: RequestType + Serialize + for<'a> Deserialize<'a>,
    {
        self.stream.send_message(request)?;
        Ok(self.stream.pull_message_from_stream()?)
    }

    pub fn transfer_admin_request<R>(&mut self, request: R) -> Result<R::Response, StreamIOError>
    where
        R: RequestType + Serialize + for<'a> Deserialize<'a>,
        ServerRequest: From<R>,
    {
        self.stream.send_message(ServerRequest::from(request))?;
        self.stream.pull_admin_response()
    }

    pub fn send_message(&mut self, message: Message) -> Result<(), StreamIOError> {
        Ok(self.stream.send_message(message)?)
    }

    pub fn receive_message(&mut self) -> Result<Message, StreamIOError> {
        Ok(self.stream.pull_message_from_stream()?)
    }

    pub fn disconnect(self) -> DisconnectedClient<T> {
        DisconnectedClient {
            config: self.config,
        }
    }
}
