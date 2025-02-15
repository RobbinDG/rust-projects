use crate::dispatcher::RequestDispatcher;
use crate::queue_store::QueueStore;
use crate::request_worker::RequestWorker;
use backend::stream_io::StreamIO;
use log::{error, info};
use std::net::SocketAddr;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::io;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

pub struct ConnectionManager {
    listener: TcpListener,
    queues: Arc<Mutex<QueueStore>>,
    setup_connections: Mutex<Vec<(SocketAddr, JoinHandle<()>)>>,
    admin_connections: Mutex<Vec<(SocketAddr, JoinHandle<StreamIO>, Sender<()>)>>,
}

impl ConnectionManager {
    pub fn new(listener: TcpListener, queues: Arc<Mutex<QueueStore>>) -> Self {
        Self {
            listener,
            queues,
            setup_connections: Mutex::new(Vec::default()),
            admin_connections: Mutex::new(Vec::default()),
        }
    }

    pub async fn start(&self) {
        let dispatcher = Arc::new(RequestDispatcher::new(self.queues.clone()));
        loop {
            match self.listener.accept().await {
                Ok((stream, addr)) => {
                    info!("New client: {addr}");
                    let worker = RequestWorker::new(StreamIO::new(stream), dispatcher.clone());
                    tokio::spawn(async move {
                        let _exit_status = worker.run().await;
                    });
                    info!("connected");
                }
                Err(e) => {
                    error!("{:?}", e);
                    continue;
                }
            };

            self.check_and_join_disconnects().unwrap();
        }
    }

    pub fn check_and_join_disconnects(&self) -> io::Result<()> {
        self.setup_connections
            .lock()
            .unwrap()
            .retain(|(_, h)| !h.is_finished());
        self.admin_connections
            .lock()
            .unwrap()
            .retain(|(_, h, _)| !h.is_finished());
        Ok(())
    }
}

impl Drop for ConnectionManager {
    fn drop(&mut self) {
        for (_, _, interrupt) in &mut self.admin_connections.lock().unwrap().iter_mut() {
            // TODO handle errors
            interrupt.send(()).unwrap();
        }
    }
}
