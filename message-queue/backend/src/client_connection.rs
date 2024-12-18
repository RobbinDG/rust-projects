use crate::protocol::request::{AdminRequest, RequestError, RequestType};
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
    pipe_broken: bool,
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
                pipe_broken: false,
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
        self.push_message(request)?;
        self.pull_message()
    }

    pub fn transfer_admin_request<R>(&mut self, request: R) -> Result<R::Response, RequestError>
    where
        R: RequestType + Serialize + for<'a> Deserialize<'a>,
        AdminRequest: From<R>,
    {
        self.push_message(AdminRequest::from(request))?;
        println!("1");
        self.pull_admin_response()
    }

    pub fn push_message<R>(&mut self, message: R) -> Result<(), StreamIOError>
    where
        R: Serialize + for<'a> Deserialize<'a>,
    {
        let result = self.stream.write(&message);

        if let Err(StreamIOError::Stream(e)) = &result {
            if e.kind() == io::ErrorKind::BrokenPipe {
                self.pipe_broken = true;
            }
        }
        result
    }

    pub fn pull_message<R>(&mut self) -> Result<R, StreamIOError>
    where
        R: Serialize + for<'a> Deserialize<'a>,
    {
        let result = self.stream.read();

        if let Err(StreamIOError::Stream(e)) = &result {
            if e.kind() == io::ErrorKind::BrokenPipe {
                self.pipe_broken = true;
            }
        }
        result
    }

    pub fn pull_admin_response<R>(&mut self) -> Result<R, RequestError>
    where
        R: Serialize + for<'a> Deserialize<'a>,
    {
        match self.stream.read_encoded_result() {
            Ok(response) => Ok(response?),
            Err(err) => {
                if let StreamIOError::Stream(e) = &err {
                    if e.kind() == io::ErrorKind::BrokenPipe {
                        self.pipe_broken = true;
                    }
                }
                Err(RequestError::IO(err))
            }
        }
    }

    pub fn broken_pipe(&self) -> bool {
        self.pipe_broken
    }

    pub fn disconnect(self) -> DisconnectedClient<T> {
        DisconnectedClient {
            config: self.config,
        }
    }
}
