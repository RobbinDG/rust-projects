use backend::{ConnectedClient, DisconnectedClient};
use backend::request::SetModeResponse;
use backend::setup_request::SetupRequest;
use backend::stream_io::StreamIOError;

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
                    Self::attempt_connect(c)
                }
                Client::Connected(connected) => {
                    if connected.broken_pipe() {
                        let disconnected = connected.disconnect();
                        match disconnected.connect() {
                            Ok(c) => Client::Connected(c),
                            Err(e) => {
                                Client::Disconnected(e.server)
                            }
                        }
                    } else {
                        Client::Connected(connected)
                    }
                }
            });
            if let Client::Connected(connected) = inserted {
                return Ok(connected)
            }
        }
        Err("Couldn't connect".to_string())
    }

    fn attempt_connect(c: DisconnectedClient<String>) -> Client {
        match c.connect() {
            Ok(mut connected) => {
                match connected.transfer_request(SetupRequest::Admin) {
                    Ok(SetModeResponse::Admin) => Client::Connected(connected),
                    _ => Client::Disconnected(connected.disconnect()),
                }
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
