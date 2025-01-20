use crate::new::dispatcher::RequestDispatcher;
use crate::new::publisher_manager::PublisherManager;
use crate::new::queue_store::QueueStore;
use crate::new::request_worker::RequestWorker;
use crate::new::subscription_manager::SubscriptionManager;
use backend::protocol::SetupResponse;
use backend::stream_io::StreamIO;
use log::{error, info};
use std::net::SocketAddr;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{io, thread};
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
        // thread::spawn(|| {
        //     self.check_and_join_disconnects().unwrap();
        //
        //     thread::sleep(Duration::from_secs(1));
        // });
        loop {
            match self.listener.accept().await {
                Ok((stream, addr)) => {
                    info!("New client: {addr}");
                    // let worker = SetupWorker::new(stream);
                    let dispatcher = RequestDispatcher::new(self.queues.clone());
                    let worker = RequestWorker::new(StreamIO::new(stream), dispatcher);
                    tokio::spawn(async move {
                        let exit_status = worker.run().await;
                    });
                    // self.handle_setup(addr, worker);
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

    // fn handle_setup(&self, addr: SocketAddr, worker: SetupWorker) {
    //     let pm = self.publisher_manager.clone();
    //     let sm = self.subscription_manager.clone();
    //     let handle = tokio::spawn(async move {
    //         let (stream, termination) = worker.run().await;
    //         match termination {
    //             SetupResponse::Disconnect => info!("{} Disconnected", addr),
    //             SetupResponse::Sender => {
    //                 pm.lock().unwrap().register_publisher(stream);
    //             }
    //             SetupResponse::Receiver(queue) => {
    //                 sm.lock().unwrap().subscribe(stream, &queue);
    //             }
    //             SetupResponse::Admin => {
    //                 let (admin_worker, interrupt) = AdminWorker::new(self.queues.clone(), stream);
    //                 self.handle_admin(addr, admin_worker, interrupt);
    //             }
    //         }
    //     });
    //     self.setup_connections.lock().unwrap().push((addr, handle));
    // }

    // fn handle_admin(&self, addr: SocketAddr, worker: AdminWorker, interrupt: Sender<()>) {
    //     let admin_handle = tokio::spawn(async move {
    //         let stream = worker.run();
    //         info!("{} Disconnected", addr);
    //         stream
    //     });
    //     self.admin_connections
    //         .lock()
    //         .unwrap()
    //         .push((addr.clone(), admin_handle, interrupt));
    // }

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
