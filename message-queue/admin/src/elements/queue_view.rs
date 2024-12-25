use crate::elements::QueueTable;
use crate::elements::UIMessage;
use crate::server_connector::ServerConnector;
use backend::protocol::request::{CreateQueue, DeleteQueue, ListQueues};
use backend::protocol::{BufferAddress, BufferType};
use iced::widget::{button, column, radio, row, text_input};
use iced::Element;

pub struct QueueView {
    // Widget state
    queue_table: QueueTable,
    new_queue_text: String,
    selected_buffer_type: Option<BufferType>,
}

impl Default for QueueView {
    fn default() -> Self {
        QueueView {
            queue_table: QueueTable::new(
                ["Queue", "Senders", "Receivers", "# Messages"],
                [300, 200, 200, 200],
            ),
            new_queue_text: String::new(),
            selected_buffer_type: Some(BufferType::Queue),
        }
    }
}

impl QueueView {
    pub fn view<'a, Message>(&'a self) -> Element<'a, Message>
    where
        Message: From<UIMessage> + Clone + 'a,
    {
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
                    .on_input(|s| UIMessage::NewQueueName(s)),
                button("Create").on_press(UIMessage::CreateQueue),
                button("Refresh").on_press(UIMessage::Refresh),
            ]
            .spacing(10),
        ];
        let element: Element<UIMessage> = cols.into();
        element.map(Message::from)
    }

    pub fn update(&mut self, message: UIMessage, connector: &mut ServerConnector) {
        match message {
            UIMessage::Refresh => self.refresh(connector),
            UIMessage::NewQueueName(s) => {
                self.new_queue_text = s;
            }
            UIMessage::CreateQueue => {
                self.create(connector);
                self.refresh(connector);
            }
            UIMessage::DeleteQueue(s) => {
                self.delete(s, connector);
                self.refresh(connector);
            }
            UIMessage::SelectBufferType(t) => {
                self.selected_buffer_type = Some(t);
            }
            UIMessage::InspectBuffer(_) => {}
        }
    }

    fn delete(&mut self, s: BufferAddress, connector: &mut ServerConnector) {
        if let Ok(client) = connector.client() {
            if let Err(_) = client.transfer_admin_request(DeleteQueue { queue_name: s }) {}
        }
    }

    fn create(&mut self, connector: &mut ServerConnector) {
        if let Ok(client) = connector.client() {
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

    fn refresh(&mut self, connector: &mut ServerConnector) {
        if let Ok(client) = connector.client() {
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
