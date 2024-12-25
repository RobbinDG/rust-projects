mod admin_worker;
mod buffer_type_manager;
mod buffer_processor;
mod connection_manager;
pub mod message_queue;
mod queue_manager;
mod request_handler;
mod server;
mod setup_worker;
mod topic;
mod topic_manager;
mod topic_processor;
mod buffer_manager;
mod buffer_interface;
mod server_error;

use server::Server;
use std::net::TcpListener;

fn main() {
    env_logger::init();
    let socket_listener = TcpListener::bind("127.0.0.1:1234").unwrap();
    let server = Server::new(socket_listener);
    server.run();
}
