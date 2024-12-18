use backend::protocol::{SetupRequest, SetupResponse};
use backend::stream_io::{StreamIO, StreamIOError};
use log::{debug, error, info};
use std::io::ErrorKind;
use std::net::TcpStream;

pub struct SetupWorker {
    stream: StreamIO,
}

impl SetupWorker {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream: StreamIO::new(stream),
        }
    }

    pub fn run(mut self) -> (StreamIO, SetupResponse) {
        info!("worker started");
        if let Err(_) = self.stream.set_timeout(None) {
            return (self.stream, SetupResponse::Disconnect);
        }

        let request: Result<SetupRequest, String> = match self.stream.read() {
            Ok(buf) => buf,
            Err(err) => {
                match err {
                    StreamIOError::Stream(e) => {
                        return match e.kind() {
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
                        return (self.stream, SetupResponse::Disconnect);
                    }
                }
            }
        };

        let promotion = match request {
            Ok(r) => {
                info!("Promoting to {r:?}");
                match r {
                    SetupRequest::Admin => SetupResponse::Admin,
                    SetupRequest::Sender(q) => SetupResponse::Sender(q),
                    SetupRequest::Receiver(q) => SetupResponse::Receiver(q),
                }
            }
            Err(e) => {
                error!("{:?}", e);
                SetupResponse::Disconnect
            }
        };

        debug!("Sending promotion response.");
        if let Err(e) = self.stream.write(&promotion) {
            error!("Unhandled error: {e:?}");
            return (self.stream, SetupResponse::Disconnect);
        }

        debug!("Sent promotion response.");
        (self.stream, promotion)
    }
}
