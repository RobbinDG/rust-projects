use backend::{ConnectedClient, DisconnectedClient};

enum Client {
    Connected(ConnectedClient<String>),
    Disconnected(DisconnectedClient<String>),
}

pub struct ServerConnector {
    client: Option<Client>,
}

impl ServerConnector {
    pub fn new(initial_address: String) -> Self {
        Self {
            client: Some(Client::Disconnected(DisconnectedClient::new(
                initial_address,
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

    pub async fn connect_to(&mut self, addr: String) -> bool {
        if let Some(client) = self.client.take() {
            if let Client::Connected(c) = client {
                c.disconnect();
            }
        }
        let new_client = DisconnectedClient::new(addr);
        let client = self.client.insert(match new_client.connect().await {
            Ok(c) => Client::Connected(c),
            Err(e) => Client::Disconnected(e.server),
        });
        matches!(client, Client::Connected(_))
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
