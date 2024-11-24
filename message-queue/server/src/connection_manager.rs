use crate::request_handler::RequestHandler;
use std::net::{SocketAddr, TcpListener};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::{io, thread};
use std::sync::mpsc::Sender;
use crate::connection_worker::ConnectionWorker;

pub struct ConnectionManager
{
    listener: TcpListener,
    // queue_manager: Arc<Mutex<QueueManager>>,
    request_handler: Arc<Mutex<RequestHandler>>,
    connections: Vec<(SocketAddr, Option<JoinHandle<io::Error>>, Sender<()>)>,
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
                    let (worker, interrupt) = ConnectionWorker::new(handler, stream);
                    let handle: JoinHandle<io::Error> = thread::spawn(move || { worker.run() });
                    self.connections.push((addr, Some(handle), interrupt));
                    println!("connected");
                }
                Err(e) => {
                    println!("{:?}", e);
                    continue;
                }
            };

            self.check_and_join_disconnects();
        }
    }

    fn check_and_join_disconnects(&mut self) {
        for (addr, handle_opt, _) in &mut self.connections {
            let handle = handle_opt.take().unwrap();
            if handle.is_finished() {
                println!("{} Disconnected", addr);
                handle.join().unwrap();
            } else {
                let _ = handle_opt.insert(handle);
            }
        }
        self.connections.retain(|(_, h, _)| h.is_some());
    }
}

impl Drop for ConnectionManager {
    fn drop(&mut self) {
        for (_, handle, interrupt) in &mut self.connections {
            // TODO handle errors
            interrupt.send(()).unwrap();
            handle.take().unwrap().join().unwrap();  // Drop using "option dance"
        }
    }
}