use backend::{ConnectedClient, DisconnectedClient};

enum Client {
    Connected(ConnectedClient<String>),
    Disconnected(DisconnectedClient<String>),
}

pub struct ServerConnector {
    client: Option<Client>,
}

impl ServerConnector {
    pub fn new() -> Self {
        Self {
            client: Some(Client::Disconnected(DisconnectedClient::new(
                "127.0.0.1:1234".to_string(),
            ))),
        }
    }

    pub async fn client(&mut self) -> Result<&mut ConnectedClient<String>, String> {
        if let Some(client) = self.client.take() {
            let inserted = self.client.insert(match client {
                Client::Disconnected(c) => Self::attempt_connect(c).await,
                Client::Connected(connected) => {
                    if connected.broken_pipe() {
                        let disconnected = connected.disconnect();
                        match disconnected.connect().await {
                            Ok(c) => Client::Connected(c),
                            Err(e) => Client::Disconnected(e.server),
                        }
                    } else {
                        Client::Connected(connected)
                    }
                }
            });
            if let Client::Connected(connected) = inserted {
                return Ok(connected);
            }
        }
        Err("Couldn't connect".to_string())
    }

    pub async fn connect_to(&mut self, addr: String) {
        if let Some(client) = self.client.take() {
            if let Client::Connected(c) = client {
                c.disconnect();
            }
        }
        let new_client = DisconnectedClient::new(addr);
        let _ = self.client.insert(match new_client.connect().await {
            Ok(c) => Client::Connected(c),
            Err(e) => Client::Disconnected(e.server),
        });
    }

    async fn attempt_connect(c: DisconnectedClient<String>) -> Client {
        match c.connect().await {
            Ok(connected) => Client::Connected(connected),
            Err(err) => Client::Disconnected(err.server),
        }
    }

    pub fn connected(&self) -> bool {
        if let Some(Client::Connected(_)) = self.client {
            return true;
        }
        false
    }
}
