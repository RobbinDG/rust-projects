mod server_connector;

use crate::server_connector::ServerConnector;
use backend::request::{CreateQueue, DeleteQueue, ListQueues};
use iced::widget::{button, column, row, text, text_input, Column, Row};
use iced::Alignment;
use std::iter::zip;

const QUEUE_VIEW_COLUMNS: [(&str, u16); 5] = [
    ("Queue", 200),
    ("Senders", 100),
    ("Receivers", 100),
    ("Message", 100),
    ("", 100),
];

#[derive(Debug, Clone)]
enum UIMessage {
    Refresh,
    NewQueueName(String),
    CreateQueue,
    DeleteQueue(String),
}

struct QueueTable {
    names: [&'static str; 4],
    widths: [u16; 4],
    content: Vec<[String; 4]>,
}

impl QueueTable {
    pub fn new(names: [&'static str; 4], widths: [u16; 4]) -> Self {
        Self {
            names,
            widths,
            content: vec![],
        }
    }

    pub fn clear(&mut self) {
        self.content.clear();
    }

    pub fn push(&mut self, row: (String, usize, usize, usize)) {
        self.content.push([
            row.0,
            row.1.to_string(),
            row.2.to_string(),
            row.3.to_string(),
        ]);
        println!("content {:?}", self.content);
    }

    pub fn view(&self) -> Column<UIMessage> {
        let header: Row<UIMessage> =
            row(zip(self.names, self.widths.clone()).map(|(name, width)| {
                text!("{}", name)
                    .width(width)
                    .align_x(Alignment::Center)
                    .into()
            }));
        let mut column: Column<UIMessage> = column![header];
        if self.content.len() <= 0 {
            column = column.push(
                text("Nothing to see...")
                    .width(500)
                    .align_x(Alignment::Center),
            );
        } else {
            for row_content in &self.content {
                let mut r: Row<UIMessage> =
                    row(zip(self.widths, row_content).map(|(w, c)| text(c).width(w).into()));
                r = r.push(
                    button("Delete").on_press(UIMessage::DeleteQueue(row_content[0].clone())),
                );
                column = column.push(r);
            }
        }
        column
    }
}

struct QueueView {
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
                    if let Ok(response) = client.transfer_admin_request(ListQueues {}) {
                        println!("ok {:?}", response);
                        self.queue_table.clear();
                        for queue_data in response {
                            self.queue_table.push(queue_data);
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
