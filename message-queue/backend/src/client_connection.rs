use crate::message::Message;
use crate::request::{RequestError, ServerRequest};
use crate::response::ServerResponse;
use std::io;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use crate::stream_io::StreamIO;

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
    stream: StreamIO<>,
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
            config: ConnectionConfig {
                address: addr,
            },
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
            })
        }
    }
}


impl<T: ToSocketAddrs + Clone> ConnectedClient<T> {
    pub fn transfer_request(&mut self, request: ServerRequest) -> Result<ServerResponse, RequestError> {
        let payload = postcard::to_allocvec(&request).unwrap();
        self.transfer_bytes(payload)
    }

    pub fn send_message(&mut self, message: Message) -> Result<(), RequestError> {
        Ok(self.stream.send_message(message)?)
    }

    pub fn receive_message(&mut self) -> Result<Message, RequestError> {
        Ok(self.stream.pull_message_from_stream()?)
    }

    pub fn transfer_bytes(&mut self, bytes: Vec<u8>) -> Result<ServerResponse, RequestError> {
        self.stream.send_message(bytes)?;
        Ok(self.stream.pull_message_from_stream()?)
    }

    pub fn disconnect(self) -> DisconnectedClient<T> {
        DisconnectedClient {
            config: self.config,
        }
    }
}
