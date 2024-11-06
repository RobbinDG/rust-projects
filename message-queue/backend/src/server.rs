use std::error::Error;
use std::io;
use std::io::{Read, Write};
use std::net::TcpStream;
use crate::response::ServerResponse;
use crate::request::ServerRequest;

pub struct DisconnectedServer {}

impl DisconnectedServer {
    pub fn new() -> DisconnectedServer {
        DisconnectedServer {}
    }

    pub fn connect(&self) -> Result<ConnectedServer, io::Error> {
        let mut stream = TcpStream::connect("localhost:1234")?;
        Ok(ConnectedServer {
            stream
        })
    }
}

pub struct ConnectedServer {
    stream: TcpStream,
}

impl ConnectedServer {
    pub fn send_request(&mut self, request: ServerRequest) -> Result<ServerResponse, Box<dyn Error>> {
        self.stream.write(request.as_payload().as_bytes())?;
        let mut buf = [0; 32];
        self.stream.read(&mut buf)?;
        let response = ServerResponse::parse(&buf)?;
        Ok(response)
    }
}
