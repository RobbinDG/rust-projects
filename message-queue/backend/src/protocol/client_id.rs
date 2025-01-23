use std::net::SocketAddr;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ClientID{
    TcpSocket(SocketAddr)
}