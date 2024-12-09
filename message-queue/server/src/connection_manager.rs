use crate::connection_worker::{AdminWorker, SetupWorker, TerminationReason};
use crate::queue_manager::QueueManager;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

pub struct ConnectionManager
{
    listener: TcpListener,
    queue_manager: Arc<Mutex<QueueManager>>,
    setup_connections: Mutex<Vec<(SocketAddr, Option<JoinHandle<(TcpStream, TerminationReason)>>, Sender<()>)>>,
    admin_connections: Mutex<Vec<(SocketAddr, Option<JoinHandle<TcpStream>>, Sender<()>)>>,
}

impl ConnectionManager
{
    pub fn new(listener: TcpListener, queue_manager: Arc<Mutex<QueueManager>>) -> Self {
        Self {
            listener,
            queue_manager,
            setup_connections: Mutex::new(Vec::default()),
            admin_connections: Mutex::new(Vec::default()),
        }
    }

    pub fn start(&self) {
        loop {
            match self.listener.accept() {
                Ok((stream, addr)) => {
                    println!("{addr}");
                    let (worker, interrupt) = SetupWorker::new(stream);
                    let handle = thread::spawn(move || { worker.run() });
                    self.setup_connections.lock().unwrap().push((addr, Some(handle), interrupt));
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
        for (addr, handle_opt, _) in &mut self.setup_connections.lock().unwrap().iter_mut() {
            let handle = handle_opt.take().unwrap();
            if handle.is_finished() {
                let (stream, termination) = handle.join().unwrap();
                println!("{:?}", termination);
                match termination {
                    TerminationReason::Disconnect => println!("{} Disconnected", addr),
                    TerminationReason::PromoteSender(queue) => {
                        self.queue_manager.lock().unwrap().connect_sender(&queue, stream);
                    }
                    TerminationReason::PromoteReceiver(queue) => {
                        self.queue_manager.lock().unwrap().connect_receiver(&queue, stream);
                    }
                    TerminationReason::PromoteAdmin => {
                        let (admin_worker, interrupt) = AdminWorker::new(self.queue_manager.clone(), stream);
                        let admin_handle = thread::spawn(move || {
                            admin_worker.run()
                        });
                        self.admin_connections.lock().unwrap().push((addr.clone(), Some(admin_handle), interrupt));
                    }
                }
            } else {
                let _ = handle_opt.insert(handle);
            }
        }
        self.setup_connections.lock().unwrap().retain(|(_, h, _)| h.is_some());
    }
}

impl Drop for ConnectionManager {
    fn drop(&mut self) {
        for (_, handle, interrupt) in &mut self.setup_connections.lock().unwrap().iter_mut() {
            // TODO handle errors
            interrupt.send(()).unwrap();
            handle.take().unwrap().join().unwrap();  // Drop using "option dance"
        }
    }
}