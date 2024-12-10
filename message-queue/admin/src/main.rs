mod server_connector;

use crate::server_connector::ServerConnector;
use backend::request::{CreateQueue, ListQueues};
use iced::widget::{button, column, row, text, text_input, Column};
use iced::Alignment;

struct QueueView {
    connector: ServerConnector,
    queues: Vec<(String, usize, usize, usize)>,
    new_queue_text: String,
    connected: bool,
}

impl Default for QueueView {
    fn default() -> Self {
        QueueView {
            connector: ServerConnector::new(),
            queues: Vec::default(),
            new_queue_text: String::new(),
            connected: false,
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
    // pub fn connect(&mut self) {
    //     if let Some(client) = self.disconnected_client.take() {
    //         match client.connect() {
    //             Ok(mut c) => {
    //                 c.transfer_request(SetupRequest::Admin).unwrap();
    //                 self.connected_client = Some(c);
    //             }
    //             Err(e) => {
    //                 self.connected_client = None;
    //                 self.disconnected_client = Some(e.server);
    //             }
    //         }
    //     }
    // }

    pub fn view(&self) -> Column<UIMessage> {
        let mut column = column![row![
            text("Queue").width(200).align_x(Alignment::Center),
            text("Senders").width(100).align_x(Alignment::Center),
            text("Receivers").width(100).align_x(Alignment::Center),
            text("Message").width(100).align_x(Alignment::Center)
        ],];
        if self.queues.len() <= 0 {
            column = column.push(
                text("Nothing to see...")
                    .width(500)
                    .align_x(Alignment::Center),
            );
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

        let mut cols = column![
            column,
            row![
                text_input("new queue name", &self.new_queue_text)
                    .on_input(UIMessage::NewQueueName),
                button("Create").on_press(UIMessage::CreateQueue),
                button("Refresh").on_press(UIMessage::Refresh),
            ],
        ];
        if !self.connected {
            cols = cols.push(text("Couldn't connect to server."));
        }
        cols
    }

    pub fn update(&mut self, message: UIMessage) {
        self.connected = true;
        match message {
            UIMessage::Refresh => match self.connector.client() {
                Ok(client) => {
                    if let Ok(response) = client.transfer_admin_request(ListQueues {}) {
                        self.queues.clear();
                        for queue_data in response {
                            self.queues.push(queue_data);
                        }
                    } else {
                        self.connected = false;
                    }
                }
                Err(_) => self.connected = false,
            },
            UIMessage::NewQueueName(s) => {
                self.new_queue_text = s;
            }
            UIMessage::CreateQueue => match self.connector.client() {
                Ok(client) => {
                    if let Err(_) = client.transfer_admin_request(CreateQueue {
                        queue_name: self.new_queue_text.clone(),
                    }) {
                        self.connected = false;
                    }
                }
                Err(_) => self.connected = false,
            },
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
