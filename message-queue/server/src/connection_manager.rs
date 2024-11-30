use crate::connection_worker::{ConnectionWorker, TerminationReason};
use crate::request_handler::RequestHandler;
use crate::QueueManager;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::thread;

pub struct ConnectionManager
{
    listener: TcpListener,
    queue_manager: Arc<Mutex<QueueManager>>,
    request_handler: Arc<Mutex<RequestHandler>>,
    connections: Mutex<Vec<(SocketAddr, Option<JoinHandle<(TcpStream, TerminationReason)>>, Sender<()>)>>,
}

impl ConnectionManager
{
    pub fn new(listener: TcpListener, queue_manager: Arc<Mutex<QueueManager>>, request_handler: RequestHandler) -> Self {
        Self {
            listener,
            queue_manager,
            request_handler: Arc::new(Mutex::new(request_handler)),
            connections: Mutex::new(Vec::default()),
        }
    }

    pub fn start(&self) {
        loop {
            match self.listener.accept() {
                Ok((stream, addr)) => {
                    println!("{addr}");
                    let handler = self.request_handler.clone();
                    let (worker, interrupt) = ConnectionWorker::new(handler, stream);
                    let handle = thread::spawn(move || { worker.run() });
                    self.connections.lock().unwrap().push((addr, Some(handle), interrupt));
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

    pub fn check_and_join_disconnects(&self) {
        for (addr, handle_opt, _) in &mut self.connections.lock().unwrap().iter_mut() {
            let handle = handle_opt.take().unwrap();
            if handle.is_finished() {
                let (stream, termination) = handle.join().unwrap();
                match termination {
                    TerminationReason::Disconnect => println!("{} Disconnected", addr),
                    TerminationReason::PromoteSender(queue) => {
                        self.queue_manager.lock().unwrap().connect_sender(&queue, stream);
                    }
                    TerminationReason::PromoteReceiver(queue) => {
                        self.queue_manager.lock().unwrap().connect_receiver(&queue, stream);
                    }
                }
            } else {
                let _ = handle_opt.insert(handle);
            }
        }
        self.connections.lock().unwrap().retain(|(_, h, _)| h.is_some());
    }
}

impl Drop for ConnectionManager {
    fn drop(&mut self) {
        for (_, handle, interrupt) in &mut self.connections.lock().unwrap().iter_mut() {
            // TODO handle errors
            interrupt.send(()).unwrap();
            handle.take().unwrap().join().unwrap();  // Drop using "option dance"
        }
    }
}