mod connection_manager;
mod request_handler;
mod connection_worker;

use crate::connection_manager::ConnectionManager;
use backend::message::Message;
use backend::message_queue::MessageQueue;
use backend::stream_io::StreamIO;
use request_handler::RequestHandler;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{io, thread};

pub struct QueueManager {
    queues: HashMap<String, (Vec<StreamIO>, MessageQueue, Vec<StreamIO>)>,
}

impl QueueManager {
    pub fn queues(&self) -> Vec<&String> {
        // TODO is a Vec a proper return type here?
        self.queues.keys().collect()
    }

    pub fn queue_exists(&self, name: &String) -> bool {
        self.queues.contains_key(name)
    }

    pub fn create(&mut self, name: String) {
        self.queues.insert(name, (Vec::default(), MessageQueue::new_empty(), Vec::default()));
    }

    pub fn process_queues(&mut self) {
        println!("Checking queues");
        for (_, (senders, queue, receivers)) in self.queues.iter_mut() {
            for sender in senders {
                match sender.pull_message_from_stream() {
                    Ok(message) => {
                        println!("{:?}", message);
                        queue.put(message)
                    }
                    Err(_) => { continue; }
                }
            }

            if let Some(recipient) = receivers.get_mut(0) {
                Self::empty_queue_to_stream(queue, recipient);
            }
        }
    }

    fn empty_queue_to_stream(queue: &mut MessageQueue, recipient: &mut StreamIO) {
        while let Some(message) = queue.pop() {
            recipient.send_message(message).unwrap()
        }
    }

    fn pull_message_from_stream(sender: &mut TcpStream) -> Result<Message, io::Error> {
        let mut buf = [0; 32];
        sender.read(&mut buf)?;
        sender.flush()?;
        let message: Message = postcard::from_bytes(&buf).unwrap();
        Ok(message)
    }

    pub fn connect_sender(&mut self, queue_name: &String, stream: TcpStream) {
        println!("connecting");
        if let Some((senders, _, _)) = self.queues.get_mut(queue_name) {
            senders.push(StreamIO::new(stream));
        }
    }

    pub fn connect_receiver(&mut self, queue_name: &String, stream: TcpStream) {
        if let Some((_, _, recipients)) = self.queues.get_mut(queue_name) {
            recipients.push(StreamIO::new(stream));
        }
    }
}

pub struct Server {
    queue_manager: Arc<Mutex<QueueManager>>,
    connection_manager: ConnectionManager,
}

impl Server {
    pub fn new(tcp_listener: TcpListener) -> Self {
        let queue_manager = Arc::new(Mutex::new(QueueManager { queues: HashMap::default() }));
        let request_handler = RequestHandler::new(queue_manager.clone());
        let connection_manager = ConnectionManager::new(tcp_listener, queue_manager.clone(), request_handler);
        Self {
            queue_manager,
            connection_manager,
        }
    }

    pub fn run(self) {
        let mut cm = Arc::new(self.connection_manager);
        let cm1 = cm.clone();
        thread::spawn(move || {
            loop {
                {
                    self.queue_manager.lock().unwrap().process_queues();
                }
                cm1.check_and_join_disconnects();

                thread::sleep(Duration::from_secs(1));
            }
        });
        cm.start()
    }
}

fn main() {
    let socket_listener = TcpListener::bind("localhost:1234").unwrap();
    let server = Server::new(socket_listener);
    server.run();
}
