mod admin_worker;
mod buffer_manager;
mod buffer_processor;
mod connection_manager;
mod message_buffer;
pub mod message_queue;
mod queue_manager;
mod request_handler;
mod server;
mod setup_worker;
mod stream_worker;
mod topic;
mod topic_manager;
mod topic_processor;

use server::Server;
use std::net::TcpListener;

fn main() {
    let socket_listener = TcpListener::bind("localhost:1234").unwrap();
    let server = Server::new(socket_listener);
    server.run();
}
