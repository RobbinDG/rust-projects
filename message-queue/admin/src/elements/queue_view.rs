use crate::elements::QueueTable;
use crate::elements::UIMessage;
use crate::server_connector::ServerConnector;
use backend::protocol::request::{CreateQueue, DeleteQueue, ListQueues};
use backend::protocol::BufferAddress;
use iced::widget::{button, column, row, text, text_input, Column};

pub struct QueueView {
    connector: ServerConnector,
    queue_table: QueueTable,
    new_queue_text: String,
}

impl Default for QueueView {
    fn default() -> Self {
        QueueView {
            connector: ServerConnector::new(),
            queue_table: QueueTable::new(
                ["Queue", "Senders", "Receivers", "Message"],
                [300, 200, 200, 200],
            ),
            new_queue_text: String::new(),
        }
    }
}

impl QueueView {
    pub fn view(&self) -> Column<UIMessage> {
        let mut cols = column![
            self.queue_table.view(),
            row![
                text_input("new queue name", &self.new_queue_text)
                    .on_input(UIMessage::NewQueueName),
                button("Create").on_press(UIMessage::CreateQueue),
                button("Refresh").on_press(UIMessage::Refresh),
            ],
        ];
        if !self.connector.connected() {
            cols = cols.push(text("Couldn't connect to server."));
        }
        cols
    }

    pub fn update(&mut self, message: UIMessage) {
        match message {
            UIMessage::Refresh => {
                if let Ok(client) = self.connector.client() {
                    match client.transfer_admin_request(ListQueues {}) {
                        Ok(response) => {
                            self.queue_table.clear();
                            for queue_data in response {
                                self.queue_table.push(queue_data);
                            }
                        }
                        Err(e) => {
                            println!("err: {e:?}");
                        }
                    }
                }
            }
            UIMessage::NewQueueName(s) => {
                self.new_queue_text = s;
            }
            UIMessage::CreateQueue => {
                if let Ok(client) = self.connector.client() {
                    if let Err(_) = client.transfer_admin_request(CreateQueue {
                        queue_address: BufferAddress::new(self.new_queue_text.clone()),
                    }) {}
                }
            }
            UIMessage::DeleteQueue(s) => {
                if let Ok(client) = self.connector.client() {
                    if let Err(_) = client.transfer_admin_request(DeleteQueue {
                        queue_name: BufferAddress::new(s),
                    }) {}
                }
            }
        }
    }
}
