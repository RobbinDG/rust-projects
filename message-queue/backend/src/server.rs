use crate::request::{RequestError, ServerRequest};
use crate::response::ServerResponse;
use std::io;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};

pub struct ServerConfig<T>
where
    T: ToSocketAddrs + Clone,
{
    address: T,
}
pub struct DisconnectedServer<T>
where
    T: ToSocketAddrs + Clone,
{
    server_config: ServerConfig<T>,
}
pub struct ConnectedServer<T>
where
    T: ToSocketAddrs + Clone,
{
    server_config: ServerConfig<T>,
    stream: TcpStream,
}

pub struct ConnectionError<T>
where
    T: ToSocketAddrs + Clone,
{
    pub error_body: io::Error,
    pub server: DisconnectedServer<T>,
}


impl<T: ToSocketAddrs + Clone> DisconnectedServer<T> {
    pub fn new(addr: T) -> DisconnectedServer<T> {
        DisconnectedServer {
            server_config: ServerConfig {
                address: addr,
            },
        }
    }

    pub fn connect(self) -> Result<ConnectedServer<T>, ConnectionError<T>> {
        match TcpStream::connect(&self.server_config.address) {
            Ok(stream) => Ok(ConnectedServer {
                server_config: self.server_config,
                stream,
            }),
            Err(e) => Err(ConnectionError {
                error_body: e,
                server: self,
            })
        }
    }
}


impl<T: ToSocketAddrs + Clone> ConnectedServer<T> {
    pub fn send_request(&mut self, request: ServerRequest) -> Result<ServerResponse, RequestError> {
        self.stream.write(request.as_payload().as_bytes())?;
        let mut buf = [0; 32];
        self.stream.read(&mut buf)?;
        let response = ServerResponse::parse(&buf)?;
        Ok(response)
    }

    pub fn disconnect(self) -> DisconnectedServer<T> {
        DisconnectedServer {
            server_config: self.server_config,
        }
    }
}
