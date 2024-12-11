use std::sync::{Arc, Mutex};
use std::net::TcpStream;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;
use std::io::{ErrorKind, Write};
use backend::request::{RequestError, AdminRequest};
use postcard::to_allocvec;
use crate::queue_manager::QueueManager;
use crate::request_handler::RequestHandler;
use crate::stream_worker::StreamWorker;

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

            let request: Result<AdminRequest, postcard::Error> = postcard::from_bytes(&buf);
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

    fn handle_request(&mut self, req: AdminRequest) -> Result<Vec<u8>, RequestError> {
        Ok(match req {
            AdminRequest::ListQueues(r) => {
                to_allocvec(&r.handle_request(self.queue_manager.clone())?)
            }
            AdminRequest::CheckQueue(r) => {
                to_allocvec(&r.handle_request(self.queue_manager.clone())?)
            }
            AdminRequest::CreateQueue(r) => {
                to_allocvec(&r.handle_request(self.queue_manager.clone())?)
            }
        }?)
    }
}