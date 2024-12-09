use crate::queue_manager::QueueManager;
use crate::request_handler::RequestHandler;
use backend::request::{RequestError, ServerRequest};
use backend::setup_request::SetupRequest;
use postcard::to_allocvec;
use std::io;
use std::io::{ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug)]
pub enum TerminationReason {
    Disconnect,
    PromoteAdmin,
    PromoteSender(String),
    PromoteReceiver(String),
}

impl TerminationReason {
    pub fn as_response(&self) -> String {
        match self {
            TerminationReason::Disconnect => "Disconnecting".to_string(),
            TerminationReason::PromoteAdmin => "Setting up admin connection".to_string(),
            TerminationReason::PromoteSender(q) => {
                format!("Setting up sender connection for queue {q}")
            }
            TerminationReason::PromoteReceiver(q) => {
                format!("Setting up receiver connection for queue {q}")
            }
        }
    }
}

trait StreamWorker {
    fn get_stream(&mut self) -> &mut TcpStream;

    fn init(&mut self, duration: Option<Duration>) -> io::Result<()> {
        self.get_stream().set_read_timeout(duration)?;
        self.get_stream().set_write_timeout(duration)
    }

    fn read(&mut self) -> io::Result<[u8; 32]> {
        let mut buf = [0; 32];
        println!("a");
        self.get_stream().read(&mut buf)?;
        println!("b");
        self.get_stream().flush()?;
        println!("c");
        Ok(buf)
    }
}

pub struct SetupWorker {
    stream: TcpStream,
    interrupt: Receiver<()>,
}

impl StreamWorker for SetupWorker {
    fn get_stream(&mut self) -> &mut TcpStream {
        &mut self.stream
    }
}

impl SetupWorker {
    pub fn new(stream: TcpStream) -> (Self, Sender<()>) {
        let (tx, rx) = channel();
        (
            Self {
                stream,
                interrupt: rx,
            },
            tx,
        )
    }

    pub fn run(mut self) -> (TcpStream, TerminationReason) {
        println!("worker started");
        if let Err(_) = self.init(None) {
            return (self.stream, TerminationReason::Disconnect);
        }

        self.get_stream().flush().unwrap();

        let buf = match self.read() {
            Ok(buf) => buf,
            Err(err) => {
                return match err.kind() {
                    // According to the docs: `Interrupted` means `read` should be retried.
                    ErrorKind::Interrupted | ErrorKind::TimedOut | ErrorKind::WouldBlock => {
                        (self.stream, TerminationReason::Disconnect)
                    }
                    // Any other error is due to external circumstances.
                    _ => {
                        println!("disconnect or something {}", err.kind());
                        (self.stream, TerminationReason::Disconnect)
                    }
                };
            }
        };

        let request: Result<SetupRequest, postcard::Error> = postcard::from_bytes(&buf);
        if let Err(e) = &request {
            println!("Received {:?}", e.to_string());
        }

        let promotion = match request {
            Ok(r) => match r {
                SetupRequest::Admin => TerminationReason::PromoteAdmin,
                SetupRequest::Sender(q) => TerminationReason::PromoteSender(q),
                SetupRequest::Receiver(q) => TerminationReason::PromoteReceiver(q),
            },
            Err(e) => {
                println!("{:?}", e);
                TerminationReason::Disconnect
            }
        };

        let payload = to_allocvec(&promotion.as_response()).unwrap();
        if let Err(_) = self.stream.write_all(&payload) {
            return (self.stream, TerminationReason::Disconnect);
        }
        println!("written");

        (self.stream, promotion)
    }
}

pub struct AdminWorker {
    queue_manager: Arc<Mutex<QueueManager>>,
    stream: TcpStream,
    interrupt_channel: Receiver<()>,
}

impl StreamWorker for AdminWorker {
    fn get_stream(&mut self) -> &mut TcpStream {
        &mut self.stream
    }
}

impl AdminWorker {
    pub fn new(queue_manager: Arc<Mutex<QueueManager>>, stream: TcpStream) -> (Self, Sender<()>) {
        let (tx, rx) = channel();
        (
            Self {
                queue_manager,
                stream,
                interrupt_channel: rx,
            },
            tx,
        )
    }

    pub fn run(mut self) -> TcpStream {
        println!("worker started");
        if let Err(_) = self.init(Some(Duration::from_secs(1))) {
            return self.stream;
        }

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
                            return self.stream;
                        }
                    }
                }
            };

            let request: Result<ServerRequest, postcard::Error> = postcard::from_bytes(&buf);
            let a = request
                .map_err(|e| RequestError::Internal(e.to_string()))
                .and_then(|r| self.handle_request(r))
                .map_err(|e| e.to_string());

            let payload = to_allocvec(&a).unwrap();
            if let Err(err) = self.stream.write_all(&payload) {
                match err.kind() {
                    // TODO I'm not sure whether this is the right course of
                    //  action on a write timeout. We could also drop the connection.
                    ErrorKind::TimedOut => continue,
                    ErrorKind::WouldBlock => continue,
                    _ => return self.stream,
                }
            }
            println!("written");
        }
    }

    fn handle_request(&mut self, req: ServerRequest) -> Result<Vec<u8>, RequestError> {
        Ok(match req {
            ServerRequest::ListQueues(r) => {
                to_allocvec(&r.handle_request(self.queue_manager.clone())?)
            }
            ServerRequest::CheckQueue(r) => {
                to_allocvec(&r.handle_request(self.queue_manager.clone())?)
            }
            ServerRequest::CreateQueue(r) => {
                to_allocvec(&r.handle_request(self.queue_manager.clone())?)
            }
        }?)
    }
}
