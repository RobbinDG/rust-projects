use crate::elements::connection_interface::ConnectionInterface;
use crate::elements::QueueTable;
use crate::elements::UIMessage;
use crate::server_connector::ServerConnector;
use backend::protocol::request::{CreateQueue, DeleteQueue, ListQueues};
use backend::protocol::{BufferAddress, BufferType};
use iced::widget::{
    button, column, container, radio, row, text, text_input, vertical_space, Column,
};
use iced::{Alignment, Length};

pub struct QueueView {
    // Widget state
    connector: ServerConnector,
    queue_table: QueueTable,
    new_queue_text: String,
    selected_buffer_type: Option<BufferType>,

    // Sub-widgets
    connection_interface: ConnectionInterface,
}

impl Default for QueueView {
    fn default() -> Self {
        QueueView {
            connector: ServerConnector::new(),
            queue_table: QueueTable::new(
                ["Queue", "Senders", "Receivers", "# Messages"],
                [300, 200, 200, 200],
            ),
            new_queue_text: String::new(),
            selected_buffer_type: Some(BufferType::Queue),
            connection_interface: ConnectionInterface::new(),
        }
    }
}

impl QueueView {
    pub fn view(&self) -> Column<UIMessage> {
        let placeholder = format!(
            "New {} name",
            match self.selected_buffer_type {
                Some(BufferType::Topic) => "topic",
                _ => "queue",
            }
        );
        let mut cols = column![
            self.queue_table.view().height(500),
            row![
                radio(
                    "Queue",
                    BufferType::Queue,
                    self.selected_buffer_type,
                    UIMessage::SelectBufferType
                ),
                radio(
                    "Topic",
                    BufferType::Topic,
                    self.selected_buffer_type,
                    UIMessage::SelectBufferType
                ),
                text_input(placeholder.as_str(), &self.new_queue_text)
                    .on_input(UIMessage::NewQueueName),
                button("Create").on_press(UIMessage::CreateQueue),
                button("Refresh").on_press(UIMessage::Refresh),
            ]
            .spacing(10),
            vertical_space(),
            self.connection_interface.view(),
        ];
        cols = cols.spacing(2).padding(10);
        if !self.connector.connected() {
            cols = cols.push(
                container(
                    text("Couldn't connect to server.")
                        .width(Length::Fill)
                        .align_x(Alignment::Center),
                )
                .width(Length::Fill)
                .align_x(Alignment::Center)
                .padding(10)
                .style(container::rounded_box),
            );
        }
        cols
    }

    pub fn update(&mut self, message: UIMessage) {
        match message {
            UIMessage::Refresh => self.refresh(),
            UIMessage::NewQueueName(s) => {
                self.new_queue_text = s;
            }
            UIMessage::CreateQueue => {
                self.create();
                self.refresh();
            }
            UIMessage::DeleteQueue(s) => {
                self.delete(s);
                self.refresh();
            }
            UIMessage::SelectBufferType(t) => {
                self.selected_buffer_type = Some(t);
            }
            UIMessage::ConnectionUpdated(m) => self.connection_interface.update(m, &self.connector),
        }
    }

    fn delete(&mut self, s: BufferAddress) {
        if let Ok(client) = self.connector.client() {
            if let Err(_) = client.transfer_admin_request(DeleteQueue { queue_name: s }) {}
        }
    }

    fn create(&mut self) {
        if let Ok(client) = self.connector.client() {
            if let Err(_) = client.transfer_admin_request(CreateQueue {
                queue_address: match self.selected_buffer_type {
                    Some(BufferType::Queue) => {
                        BufferAddress::new_queue(self.new_queue_text.clone())
                    }
                    Some(BufferType::Topic) => {
                        BufferAddress::new_topic(self.new_queue_text.clone())
                    }
                    _ => todo!("No buffer selected"),
                },
            }) {}
        }
    }

    fn refresh(&mut self) {
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
}
