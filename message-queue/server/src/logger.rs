use backend::protocol::message::{Message, TTL};
use backend::protocol::queue_id::{NewQueueId, QueueId};
use backend::protocol::request::{CreateQueue, Publish, SupportedRequest};
use backend::protocol::routing_key::{DLXPreference, RoutingKey};
use backend::protocol::{Request, UserQueueProperties};
use backend::stream_io::StreamIO;
use log::{LevelFilter, Metadata, Record};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::runtime::Handle;
use tokio::sync::mpsc::{channel, Sender};
use tokio::task;

pub struct QueueLogger {
    log_sender: Sender<Message>,
    create_sender: Sender<CreateQueue>,
    currently_logging: Arc<AtomicBool>,
}

impl QueueLogger {
    pub fn new(address: &'static str) -> Self {
        let (tx, mut rx) = channel::<Message>(2);
        let (tx_c, mut rx_c) = channel::<CreateQueue>(2);
        let currently_logging = Arc::new(AtomicBool::new(false));
        let currently_logging2 = currently_logging.clone();
        let currently_logging3 = currently_logging.clone();

        tokio::spawn(async move {
            currently_logging3.store(true, Ordering::Relaxed);
            let stream = TcpStream::connect(address).await.unwrap();
            let mut stream = StreamIO::new(stream);
            currently_logging3.store(false, Ordering::Relaxed);

            while let Some(msg) = rx_c.recv().await {
                currently_logging3.store(true, Ordering::Relaxed);
                stream
                    .write_encode(&SupportedRequest::CreateQueue(msg))
                    .await
                    .unwrap();
                stream
                    .read_encoded_result::<<CreateQueue as Request>::Response>()
                    .await
                    .unwrap()
                    .unwrap();
                currently_logging3.store(false, Ordering::Relaxed);
            }
        });

        tokio::spawn(async move {
            currently_logging.store(true, Ordering::Relaxed);
            let stream = TcpStream::connect(address).await.unwrap();
            let mut stream = StreamIO::new(stream);
            currently_logging.store(false, Ordering::Relaxed);

            while let Some(msg) = rx.recv().await {
                currently_logging.store(true, Ordering::Relaxed);
                stream
                    .write_encode(&SupportedRequest::Publish(Publish { message: msg }))
                    .await
                    .unwrap();
                stream
                    .read_encoded_result::<<Publish as Request>::Response>()
                    .await
                    .unwrap()
                    .unwrap()
                    .unwrap();
                currently_logging.store(false, Ordering::Relaxed);
            }
        });

        QueueLogger {
            log_sender: tx,
            create_sender: tx_c,
            currently_logging: currently_logging2,
        }
    }
}

impl log::Log for QueueLogger {
    fn enabled(&self, _: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.currently_logging.load(Ordering::Relaxed) {
            // Ignoring logs if we are currently logging.
            return;
        }

        let file = record.file().unwrap_or_else(|| "unknown").to_string();
        let level = record.level().to_string();
        let queue = QueueId::Topic("logs".to_string(), file, level);
        let new_queue = NewQueueId::from(queue.clone());

        task::block_in_place(|| {
            Handle::current()
                .block_on(async move {
                    self.create_sender.send(CreateQueue {
                        queue_address: new_queue,
                        properties: UserQueueProperties { is_dlx: false, dlx: None },
                    }).await
                })
                .unwrap()
        });

        let message = Message {
            payload: record.args().to_string().into(),
            routing_key: RoutingKey {
                id: queue,
                dlx: DLXPreference::Default,
            },
            ttl: TTL::Permanent,
        };
        task::block_in_place(|| {
            Handle::current()
                .block_on(async move { self.log_sender.send(message).await })
                .unwrap()
        });
    }

    fn flush(&self) {}
}

static LOGGER: once_cell::sync::Lazy<QueueLogger> =
    once_cell::sync::Lazy::new(|| QueueLogger::new("127.0.0.1:1234"));

pub fn init() {
    log::set_logger(&*LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Info))
        .expect("Failed to initialize logger");
}
