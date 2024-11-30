use iced::widget::{button, column, text, Column};
use backend::{ConnectedClient, DisconnectedClient};
use backend::request::ServerRequest;

struct QueueList {
    pub queues: Vec<String>,
}

struct QueueView {
    connected_client: Option<ConnectedClient<String>>,
    disconnected_client: Option<DisconnectedClient<String>>,
    queues: Vec<String>,
}

impl Default for QueueView {
    fn default() -> Self {
        QueueView {
            connected_client: None,
            disconnected_client: Some(DisconnectedClient::new("localhost:1234".to_string())),
            queues: Vec::default(),
        }
    }
}

#[derive(Debug, Clone)]
struct RefreshMessage {}

impl QueueView {
    pub fn connect(&mut self) {
        if let Some(client) = self.disconnected_client.take() {
            match client.connect() {
                Ok(c) => {
                    self.connected_client = Some(c);
                }
                Err(e) => {
                    self.connected_client = None;
                    self.disconnected_client = Some(e.server);
                }
            }
        }
    }

    pub fn view(&self) -> Column<RefreshMessage> {
        let mut column = column![];
        for queue in &self.queues {
            column = column.push(text(queue));
        }

        column![
            column,
            button("Refresh").on_press(RefreshMessage {}),
        ]
    }

    pub fn update(&mut self, message: RefreshMessage) {
        self.connect();
        if let Some(client) = &mut self.connected_client {
            if let Ok(response) = client.transfer_request(ServerRequest::ListQueues) {
                self.queues.push(response.payload);
            }
        }
    }
}

fn main() -> iced::Result {
    iced::run("Message Queue Admin Panel", QueueView::update, QueueView::view)
}