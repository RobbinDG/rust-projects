mod connection_manager;
mod server;
mod queue;
pub mod router;
pub mod queue_store;
pub mod request_worker;
pub mod dispatcher;
mod request_handler;
mod subscription_manager;

use server::Server;
use std::error::Error;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let socket_listener = match TcpListener::bind("127.0.0.1:1234").await {
        Ok(listener) => listener,
        Err(error) => panic!("{}", error),
    };

    let server = Server::new(socket_listener);
    server.run().await
}
