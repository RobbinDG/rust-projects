use crate::queue_manager::QueueManager;
use crate::request_handler::RequestHandler;
use backend::request::{RequestError, ServerRequest, SetModeResponse};
use backend::setup_request::SetupRequest;
use postcard::to_allocvec;
use std::io;
use std::io::{ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

trait StreamWorker {
    fn get_stream(&mut self) -> &mut TcpStream;

    fn init(&mut self, duration: Option<Duration>) -> io::Result<()> {
        self.get_stream().set_read_timeout(duration)?;
        self.get_stream().set_write_timeout(duration)
    }

    fn read(&mut self) -> io::Result<[u8; 32]> {
        let mut buf = [0; 32];
        self.get_stream().read(&mut buf)?;
        self.get_stream().flush()?;
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

    pub fn run(mut self) -> (TcpStream, SetModeResponse) {
        println!("worker started");
        if let Err(_) = self.init(None) {
            return (self.stream, SetModeResponse::Disconnect);
        }

        self.get_stream().flush().unwrap();

        let buf = match self.read() {
            Ok(buf) => buf,
            Err(err) => {
                return match err.kind() {
                    // According to the docs: `Interrupted` means `read` should be retried.
                    ErrorKind::Interrupted | ErrorKind::TimedOut | ErrorKind::WouldBlock => {
                        (self.stream, SetModeResponse::Disconnect)
                    }
                    // Any other error is due to external circumstances.
                    _ => {
                        println!("disconnect or something {}", err.kind());
                        (self.stream, SetModeResponse::Disconnect)
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
                SetupRequest::Admin => SetModeResponse::Admin,
                SetupRequest::Sender(q) => SetModeResponse::Sender(q.replace("\n", "")),
                SetupRequest::Receiver(q) => SetModeResponse::Receiver(q.replace("\n", "")),
            },
            Err(e) => {
                println!("{:?}", e);
                SetModeResponse::Disconnect
            }
        };

        let payload = to_allocvec(&promotion).unwrap();
        if let Err(_) = self.stream.write_all(&payload) {
            return (self.stream, SetModeResponse::Disconnect);
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

            println!("to send {:?}", a);
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
