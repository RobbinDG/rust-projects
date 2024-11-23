use crate::RequestHandler;
use backend::request::{RequestError, ServerRequest};
use backend::response::ServerResponse;
use backend::status_code::Status;
use postcard::to_allocvec;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

pub struct ConnectionManager
{
    listener: TcpListener,
    // queue_manager: Arc<Mutex<QueueManager>>,
    request_handler: Arc<Mutex<RequestHandler>>,
    connections: Vec<(SocketAddr)>,
}

impl ConnectionManager
{
    pub fn new(listener: TcpListener, request_handler: RequestHandler) -> Self {
        Self {
            listener,
            // queue_manager: Arc::new(Mutex::new(queue_manager)),
            request_handler: Arc::new(Mutex::new(request_handler)),
            connections: Vec::default(),
        }
    }

    pub fn start(mut self) {
        loop {
            match self.listener.accept() {
                Ok((mut stream, addr)) => {
                    println!("{addr}");
                    let handler = self.request_handler.clone();
                    let handle: JoinHandle<Result<(), RequestError>> = thread::spawn(move || {
                        loop {
                            let mut buf = [0; 32];
                            stream.read(&mut buf)?;
                            let request: Result<ServerRequest, postcard::Error> = postcard::from_bytes(&buf);
                            println!("Received {:?}", request);

                            stream.flush()?;

                            let response = match request {
                                Ok(r) => {
                                    println!("obtaining");
                                    let x = handler.lock().expect("mutex poisoned").handle_request(r)?;
                                    println!("releasing");
                                    x
                                },
                                Err(e) => {
                                    println!("{:?}", e);
                                    ServerResponse::from_status(Status::UnknownCommand)
                                }
                            };

                            let payload = to_allocvec(&response).unwrap();
                            stream.write_all(&payload)?;
                            println!("written");
                        }
                    });
                    self.connections.push((addr));
                    println!("connected");
                }
                Err(e) => {
                    println!("{:?}", e);
                    continue;
                }
            };
        }
    }
}