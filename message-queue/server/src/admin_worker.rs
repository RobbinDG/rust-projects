use crate::buffer_manager::BufferManager;
use crate::request_handler::RequestHandler;
use backend::protocol::request::{AdminRequest, RequestError};
use backend::protocol::{ResponseError, SetupRequest};
use backend::stream_io::{StreamIO, StreamIOError};
use log::{debug, error, info};
use postcard::to_allocvec;
use std::io::{ErrorKind, Write};
use std::net::TcpStream;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub struct AdminWorker {
    queue_manager: Arc<Mutex<BufferManager>>,
    stream: StreamIO,
    interrupt_channel: Receiver<()>,
}


impl AdminWorker {
    pub fn new(queue_manager: Arc<Mutex<BufferManager>>, stream: StreamIO) -> (Self, Sender<()>) {
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

    pub fn run(mut self) -> StreamIO {
        info!("Started admin worker");
        if let Err(_) = self.stream.set_timeout(Some(Duration::from_secs(1))) {
            return self.stream;
        }

        loop {
            let request: Result<AdminRequest, ResponseError> = match self.stream.read() {
                Ok(buf) => Ok(buf),
                Err(err) => {
                    match err {
                        StreamIOError::Stream(e) => match e.kind() {
                            // According to the docs: `Interrupted` means `read` should be retried.
                            ErrorKind::Interrupted => continue,
                            ErrorKind::TimedOut => continue,
                            ErrorKind::WouldBlock => continue,
                            // Any other error is due to external circumstances.
                            _ => {
                                error!("Unexpected disconnect: {}", e.kind());
                                return self.stream;
                            }
                        },
                        _ => {
                            error!("Unhandled error: {err:?}");
                            Err(ResponseError::CommunicationFailed)
                        }
                    }
                }
            };

            let response: Result<Vec<u8>, ResponseError> = request
                .and_then(|r| self.handle_request(r));

            debug!("Sending response {:?}", response);
            if let Err(err) = self.stream.write(&response) {
                match err {
                    StreamIOError::Stream(e) => match e.kind() {
                        ErrorKind::TimedOut => continue,
                        ErrorKind::WouldBlock => continue,
                        _ => {
                            error!("Unexpected disconnect: {}", e.kind());
                            return self.stream;
                        }
                    },
                    _ => {
                        error!("Unhandled Error {err:?}");
                        continue;
                    }
                }
            }
            debug!("Response sent");
        }
    }

    fn handle_request(&mut self, req: AdminRequest) -> Result<Vec<u8>, ResponseError> {
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
            AdminRequest::DeleteQueue(r) => {
                to_allocvec(&r.handle_request(self.queue_manager.clone())?)
            }
        }?)
    }
}
