use iced::futures::executor::block_on;
use backend::protocol::{SetupRequest, SetupResponse};
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

    pub fn client(&mut self) -> Result<&mut ConnectedClient<String>, String> {
        if let Some(client) = self.client.take() {
            let inserted = self.client.insert(match client {
                Client::Disconnected(c) => Self::attempt_connect(c),
                Client::Connected(connected) => {
                    if connected.broken_pipe() {
                        let disconnected = connected.disconnect();
                        match block_on(disconnected.connect()) {
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

    pub fn connect_to(&mut self, addr: String) {
        if let Some(client) = self.client.take() {
            if let Client::Connected(c) = client {
                c.disconnect();
            }
        }
        let new_client = DisconnectedClient::new(addr);
        let _ = self.client.insert(match block_on(new_client.connect()) {
            Ok(c) => Client::Connected(c),
            Err(e) => Client::Disconnected(e.server),
        });
    }

    fn attempt_connect(c: DisconnectedClient<String>) -> Client {
        let rt = tokio::runtime::Runtime::new().unwrap();
        match rt.block_on(c.connect()) {
            Ok(mut connected) => match block_on(connected.transfer_request(SetupRequest::Admin)) {
                Ok(SetupResponse::Admin) => Client::Connected(connected),
                _ => Client::Disconnected(connected.disconnect()),
            },
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
