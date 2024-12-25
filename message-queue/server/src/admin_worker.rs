use crate::buffer_manager::BufferManager;
use crate::request_handler::RequestHandler;
use crate::server_error::ServerError;
use backend::protocol::request::AdminRequest;
use backend::protocol::{ResponseError, Status};
use backend::stream_io::{StreamIO, StreamIOError};
use log::{debug, error, info};
use postcard::to_allocvec;
use std::io::ErrorKind;
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

        while self.interrupt_channel.try_recv().is_err() {
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

            let response: Result<Vec<u8>, ResponseError> =
                request.and_then(|r| self.handle_request(r));

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
        self.stream
    }

    fn handle_request(&mut self, req: AdminRequest) -> Result<Vec<u8>, ResponseError> {
        self.handle_request2(req).unwrap_or_else(|err| {
            error!("Execution failed: {:?}", err);
            Err(ResponseError::ExecFailed(Status::Error))
        })
    }

    fn handle_request2(
        &mut self,
        req: AdminRequest,
    ) -> Result<Result<Vec<u8>, ResponseError>, ServerError> {
        match req {
            AdminRequest::ListQueues(r) => r
                .handle_request(self.queue_manager.clone())
                .map(|s| s.and_then(|v| to_allocvec(&v).map_err(Into::into))),
            AdminRequest::CheckQueue(r) => r
                .handle_request(self.queue_manager.clone())
                .map(|s| s.and_then(|v| to_allocvec(&v).map_err(Into::into))),
            AdminRequest::CreateQueue(r) => r
                .handle_request(self.queue_manager.clone())
                .map(|s| s.and_then(|v| to_allocvec(&v).map_err(Into::into))),
            AdminRequest::DeleteQueue(r) => r
                .handle_request(self.queue_manager.clone())
                .map(|s| s.and_then(|v| to_allocvec(&v).map_err(Into::into))),
            AdminRequest::GetProperties(b) => b
                .handle_request(self.queue_manager.clone())
                .map(|s| s.and_then(|v| to_allocvec(&v).map_err(Into::into))),
        }
    }
}
