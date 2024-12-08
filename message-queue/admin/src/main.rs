use backend::request::{CreateQueue, ListQueues};
use backend::setup_request::SetupRequest;
use backend::{ConnectedClient, DisconnectedClient};
use iced::widget::{button, column, row, text, text_input, Column};
use iced::Alignment;

struct QueueView {
    connected_client: Option<ConnectedClient<String>>,
    disconnected_client: Option<DisconnectedClient<String>>,
    queues: Vec<(String, usize, usize, usize)>,
    new_queue_text: String,
}

impl Default for QueueView {
    fn default() -> Self {
        QueueView {
            connected_client: None,
            disconnected_client: Some(DisconnectedClient::new("localhost:1234".to_string())),
            queues: Vec::default(),
            new_queue_text: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
enum UIMessage {
    Refresh,
    NewQueueName(String),
    CreateQueue,
}

impl QueueView {
    pub fn connect(&mut self) {
        if let Some(client) = self.disconnected_client.take() {
            match client.connect() {
                Ok(mut c) => {
                    c.transfer_request(SetupRequest::Admin).unwrap();
                    self.connected_client = Some(c);
                }
                Err(e) => {
                    self.connected_client = None;
                    self.disconnected_client = Some(e.server);
                }
            }
        }
    }

    pub fn view(&self) -> Column<UIMessage> {
        let mut column = column![row![
            text("Queue").width(200).align_x(Alignment::Center),
            text("Senders").width(100).align_x(Alignment::Center),
            text("Receivers").width(100).align_x(Alignment::Center),
            text("Message").width(100).align_x(Alignment::Center)
        ],];
        if self.queues.len() <= 0 {
            column = column.push(text("Nothing to see...")
                .width(500)
                .align_x(Alignment::Center));
        } else {
            for (queue, senders, receivers, messages) in &self.queues {
                column = column.push(row![
                    text(queue).width(200),
                    text(senders).width(100),
                    text(receivers).width(100),
                    text(messages).width(100),
                ]);
            }
        }

        column![
            column,
            row![
                text_input("new queue name", &self.new_queue_text)
                    .on_input(UIMessage::NewQueueName),
                button("Create").on_press(UIMessage::CreateQueue),
                button("Refresh").on_press(UIMessage::Refresh),
            ],
        ]
    }

    pub fn update(&mut self, message: UIMessage) {
        match message {
            UIMessage::Refresh => {
                self.connect();
                if let Some(client) = &mut self.connected_client {
                    if let Ok(response) = client.transfer_admin_request(ListQueues {}) {
                        self.queues.clear();
                        for queue_data in response {
                            self.queues.push(queue_data);
                        }
                    }
                }
            }
            UIMessage::NewQueueName(s) => {
                self.new_queue_text = s;
            }
            UIMessage::CreateQueue => {
                self.connect();
                if let Some(client) = &mut self.connected_client {
                    client
                        .transfer_admin_request(CreateQueue {
                            queue_name: self.new_queue_text.clone(),
                        })
                        .unwrap();
                }
            }
        }
    }
}

fn main() -> iced::Result {
    iced::run(
        "Message Queue Admin Panel",
        QueueView::update,
        QueueView::view,
    )
}
