mod connection_manager;
mod new;
mod server;

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
