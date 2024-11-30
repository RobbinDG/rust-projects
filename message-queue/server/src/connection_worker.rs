use crate::request_handler::{RequestHandler, ResponseType};
use backend::request::ServerRequest;
use backend::response::ServerResponse;
use backend::status_code::Status;
use postcard::to_allocvec;
use std::io;
use std::io::{ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;


pub enum TerminationReason {
    Disconnect,
    PromoteSender(String),
    PromoteReceiver(String),
}

pub struct ConnectionWorker {
    handler: Arc<Mutex<RequestHandler>>,
    stream: TcpStream,
    interrupt_channel: Receiver<()>,
}

impl ConnectionWorker {
    pub fn new(handler: Arc<Mutex<RequestHandler>>, stream: TcpStream) -> (Self, Sender<()>) {
        let (tx, rx) = channel();
        (Self {
            handler,
            stream,
            interrupt_channel: rx,
        }, tx)
    }

    fn init(&mut self) -> io::Result<()> {
        self.stream.set_read_timeout(Some(Duration::from_secs(1)))?;
        self.stream.set_write_timeout(Some(Duration::from_secs(1)))
    }

    fn read(&mut self) -> io::Result<[u8; 32]> {
        let mut buf = [0; 32];
        self.stream.read(&mut buf)?;
        self.stream.flush()?;
        Ok(buf)
    }

    pub fn run(mut self) -> (TcpStream, TerminationReason) {
        println!("worker started");
        if let Err(_) = self.init() { return (self.stream, TerminationReason::Disconnect)}

        loop {
            let buf = match self.read() {
                Ok(buf) => buf,
                Err(err) => {
                    match err.kind() {
                        // According to the docs: `Interrupted` means `read` should be retried.
                        ErrorKind::Interrupted => continue,
                        ErrorKind::TimedOut => continue,
                        ErrorKind::WouldBlock => continue,
                        // Any other error is due to external circumstances.
                        _ => {
                            println!("disconnect or something {}", err.kind());
                            return (self.stream, TerminationReason::Disconnect)
                        },
                    }
                }
            };

            let request: Result<ServerRequest, postcard::Error> = postcard::from_bytes(&buf);
            println!("Received {:?}", request);

            let response = match request {
                Ok(r) => {
                    self.handler.lock()
                        .map_err(|_| ServerResponse::from_status(Status::Error))
                        .and_then(|mut x|
                            x.handle_request(r)
                                .map_err(|_| ServerResponse::from_status(Status::Error))
                        ).unwrap_or_else(|err| ResponseType::Response(err))
                }
                Err(e) => {
                    println!("{:?}", e);
                    ResponseType::Response(ServerResponse::from_status(Status::UnknownCommand))
                }
            };

            let mut terminate: Option<TerminationReason> = None;
            let response_msg = match response {
                ResponseType::Response(r) => r,
                ResponseType::PromoteSender(r, q) => {
                    terminate = Some(TerminationReason::PromoteSender(q));
                    r
                }
                ResponseType::PromoteReceiver(r, q) => {
                    terminate = Some(TerminationReason::PromoteReceiver(q));
                    r
                }
            };

            let payload = to_allocvec(&response_msg).unwrap();
            if let Err(err) = self.stream.write_all(&payload) {
                match err.kind() {
                    // TODO I'm not sure whether this is the right course of
                    //  action on a write timeout. We could also drop the connection.
                    ErrorKind::TimedOut => continue,
                    ErrorKind::WouldBlock => continue,
                    _ => return (self.stream, TerminationReason::Disconnect),
                }
            }
            println!("written");

            if let Some(termination) = terminate {
                return (self.stream, termination);
            }
        }
    }
}