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
                "localhost:1234".to_string(),
            ))),
        }
    }

    pub fn client(&mut self) -> Result<&mut ConnectedClient<String>, String> {
        if let Some(client) = self.client.take() {
            let inserted = self.client.insert(match client {
                Client::Disconnected(c) => {
                    match c.connect() {
                        Ok(connected) => {
                            Client::Connected(connected)
                        },
                        Err(err) => Client::Disconnected(err.server),
                    }
                }
                Client::Connected(connected) => {
                    Client::Connected(connected)
                }
            });
            if let Client::Connected(connected) = inserted {
                return Ok(connected)
            }
        }
        Err("Couldn't connect".to_string())
    }
}
