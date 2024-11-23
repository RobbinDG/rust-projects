use crate::request_handler::RequestHandler;
use std::net::{SocketAddr, TcpListener};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::{io, thread};
use crate::connection_worker::ConnectionWorker;

pub struct ConnectionManager
{
    listener: TcpListener,
    // queue_manager: Arc<Mutex<QueueManager>>,
    request_handler: Arc<Mutex<RequestHandler>>,
    connections: Vec<(SocketAddr, JoinHandle<io::Error>)>,
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
                Ok((stream, addr)) => {
                    println!("{addr}");
                    let handler = self.request_handler.clone();
                    let worker = ConnectionWorker::new(handler, stream);
                    let handle: JoinHandle<io::Error> = thread::spawn(move || { worker.run() });
                    self.connections.push((addr, handle));
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

impl Drop for ConnectionManager {
    fn drop(&mut self) {
        todo!()
    }
}