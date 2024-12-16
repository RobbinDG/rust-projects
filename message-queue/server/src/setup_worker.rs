use crate::stream_worker::StreamWorker;
use backend::protocol::{SetupRequest, SetupResponse};
use postcard::to_allocvec;
use std::io::{ErrorKind, Write};
use std::net::TcpStream;
use std::sync::mpsc::{channel, Receiver, Sender};

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

    pub fn run(mut self) -> (TcpStream, SetupResponse) {
        println!("worker started");
        if let Err(_) = self.init(None) {
            return (self.stream, SetupResponse::Disconnect);
        }

        self.get_stream().flush().unwrap();

        let buf = match self.read() {
            Ok(buf) => buf,
            Err(err) => {
                return match err.kind() {
                    // According to the docs: `Interrupted` means `read` should be retried.
                    ErrorKind::Interrupted | ErrorKind::TimedOut | ErrorKind::WouldBlock => {
                        (self.stream, SetupResponse::Disconnect)
                    }
                    // Any other error is due to external circumstances.
                    _ => {
                        println!("disconnect or something {}", err.kind());
                        (self.stream, SetupResponse::Disconnect)
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
                SetupRequest::Admin => SetupResponse::Admin,
                SetupRequest::Sender(q) => SetupResponse::Sender(q),
                SetupRequest::Receiver(q) => SetupResponse::Receiver(q),
            },
            Err(e) => {
                println!("{:?}", e);
                SetupResponse::Disconnect
            }
        };

        let payload = to_allocvec(&promotion).unwrap();
        if let Err(_) = self.stream.write_all(&payload) {
            return (self.stream, SetupResponse::Disconnect);
        }
        println!("written");

        (self.stream, promotion)
    }
}

