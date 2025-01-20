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
use crate::new::queue_store::QueueStore;

pub struct AdminWorker {
    queues: Arc<Mutex<QueueStore>>,
    stream: StreamIO,
    interrupt_channel: Receiver<()>,
}

impl AdminWorker {
    pub fn new(queues: Arc<Mutex<QueueStore>>, stream: StreamIO) -> (Self, Sender<()>) {
        let (tx, rx) = channel();
        (
            Self {
                queues,
                stream,
                interrupt_channel: rx,
            },
            tx,
        )
    }

    pub fn run(mut self) -> StreamIO {
        info!("Started admin worker");
        while self.interrupt_channel.try_recv().is_err() {
            let request: Result<AdminRequest, ResponseError> = match self.stream.try_read() {
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
            if let Err(err) = self.stream.write_encode(&response) {
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
                .handle_request(self.queues.clone())
                .map(|s| s.and_then(|v| to_allocvec(&v).map_err(Into::into))),
            AdminRequest::CheckQueue(r) => r
                .handle_request(self.queues.clone())
                .map(|s| s.and_then(|v| to_allocvec(&v).map_err(Into::into))),
            AdminRequest::CreateQueue(r) => r
                .handle_request(self.queues.clone())
                .map(|s| s.and_then(|v| to_allocvec(&v).map_err(Into::into))),
            AdminRequest::DeleteQueue(r) => r
                .handle_request(self.queues.clone())
                .map(|s| s.and_then(|v| to_allocvec(&v).map_err(Into::into))),
            AdminRequest::GetProperties(b) => b
                .handle_request(self.queues.clone())
                .map(|s| s.and_then(|v| to_allocvec(&v).map_err(Into::into))),
        }
    }
}
