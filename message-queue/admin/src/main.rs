mod server_connector;

use crate::server_connector::ServerConnector;
use backend::request::{CreateQueue, DeleteQueue, ListQueues};
use iced::widget::{button, column, row, text, text_input, Column, Row};
use iced::Alignment;

const QUEUE_VIEW_COLUMNS: [(&str, u16); 5] = [
    ("Queue", 200),
    ("Senders", 100),
    ("Receivers", 100),
    ("Message", 100),
    ("", 100),
];

struct QueueView {
    connector: ServerConnector,
    queues: Vec<(String, usize, usize, usize)>,
    new_queue_text: String,
}

impl Default for QueueView {
    fn default() -> Self {
        QueueView {
            connector: ServerConnector::new(),
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
    DeleteQueue(String),
}

impl QueueView {
    pub fn view(&self) -> Column<UIMessage> {
        let header: Row<UIMessage> = row(QUEUE_VIEW_COLUMNS.iter().map(|(name, width)| {
            text!("{}", name)
                .width(width.clone())
                .align_x(Alignment::Center)
                .into()
        }));

        let mut column: Column<UIMessage> = column![header];
        println!("{:?}", self.queues);
        if self.queues.len() <= 0 {
            column = column.push(
                text("Nothing to see...")
                    .width(500)
                    .align_x(Alignment::Center),
            );
        } else {
            for (queue, senders, receivers, messages) in &self.queues {
                column = column.push(row![
                    text(queue).width(QUEUE_VIEW_COLUMNS[0].1),
                    text(senders).width(QUEUE_VIEW_COLUMNS[1].1),
                    text(receivers).width(QUEUE_VIEW_COLUMNS[2].1),
                    text(messages).width(QUEUE_VIEW_COLUMNS[3].1),
                    button("Delete")
                        .on_press(UIMessage::DeleteQueue(queue.clone())),
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
        if !self.connector.connected() {
            cols = cols.push(text("Couldn't connect to server."));
        }
        cols
    }

    pub fn update(&mut self, message: UIMessage) {
        match message {
            UIMessage::Refresh => {
                if let Ok(client) = self.connector.client() {
                    if let Ok(response) = client.transfer_admin_request(ListQueues {}) {
                        println!("ok {:?}", response);
                        self.queues.clear();
                        for queue_data in response {
                            self.queues.push(queue_data);
                        }
                    } else {
                        println!("err");
                    }
                }
            }
            UIMessage::NewQueueName(s) => {
                self.new_queue_text = s;
            }
            UIMessage::CreateQueue => {
                if let Ok(client) = self.connector.client() {
                    if let Err(_) = client.transfer_admin_request(CreateQueue {
                        queue_name: self.new_queue_text.clone(),
                    }) {}
                }
            }
            UIMessage::DeleteQueue(s) => {
                if let Ok(client) = self.connector.client() {
                    if let Err(_) = client.transfer_admin_request(DeleteQueue { queue_name: s }) {}
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
