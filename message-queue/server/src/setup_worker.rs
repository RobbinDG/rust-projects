use backend::protocol::{SetupRequest, SetupResponse};
use backend::stream_io::{StreamIO, StreamIOError};
use log::{debug, error, info};
use std::io::ErrorKind;
use tokio::net::TcpStream;

pub struct SetupWorker {
    stream: StreamIO,
}

impl SetupWorker {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream: StreamIO::new(stream),
        }
    }

    pub async fn run(mut self) -> (StreamIO, SetupResponse) {
        info!("worker started");
        let request: SetupRequest = match self.stream.read().await {
            Ok(buf) => buf,
            Err(err) => {
                return match err {
                    StreamIOError::Stream(e) => {
                        match e.kind() {
                            // According to the docs: `Interrupted` means `read` should be retried.
                            ErrorKind::Interrupted
                            | ErrorKind::TimedOut
                            | ErrorKind::WouldBlock => (self.stream, SetupResponse::Disconnect),
                            // Any other error is due to external circumstances.
                            _ => {
                                error!("disconnect or something {}", e.kind());
                                return (self.stream, SetupResponse::Disconnect);
                            }
                        }
                    }
                    _ => {
                        error!("Unhandled error: {err:?}");
                        (self.stream, SetupResponse::Disconnect)
                    }
                };
            }
        };

        info!("Promoting to {request:?}");
        let promotion = match request {
            SetupRequest::Admin => SetupResponse::Admin,
            SetupRequest::Sender(q) => SetupResponse::Sender(q),
            SetupRequest::Receiver(q) => SetupResponse::Receiver(q),
        };

        debug!("Sending promotion response.");
        if let Err(e) = self.stream.write_encode(&promotion).await {
            error!("Unhandled error: {e:?}");
            return (self.stream, SetupResponse::Disconnect);
        }

        debug!("Sent promotion response.");
        self.stream.reset();
        (self.stream, promotion)
    }
}
