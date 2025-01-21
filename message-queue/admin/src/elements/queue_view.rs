use crate::elements::QueueTable;
use crate::elements::UIMessage;
use crate::server_connector::ServerConnector;
use backend::protocol::new::queue_id::{QueueId, QueueType};
use backend::protocol::request::{CreateQueue, ListQueues};
use iced::widget::{button, column, radio, row, text_input};
use iced::{Element, Task};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct QueueView {
    // Widget state
    queue_table: QueueTable,
    new_queue_text: String,
    selected_buffer_type: Option<QueueType>,
}

impl Default for QueueView {
    fn default() -> Self {
        QueueView {
            queue_table: QueueTable::new(
                ["Queue", "Senders", "Receivers", "# Messages"],
                [300, 200, 200, 200],
            ),
            new_queue_text: String::new(),
            selected_buffer_type: Some(QueueType::Queue),
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
                Some(QueueType::Topic) => "topic",
                _ => "queue",
            }
        );

        let cols = column![
            self.queue_table.view().height(500),
            row![
                radio(
                    "Queue",
                    QueueType::Queue,
                    self.selected_buffer_type,
                    UIMessage::SelectBufferType
                ),
                radio(
                    "Topic",
                    QueueType::Topic,
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

    pub fn update<'a, Message>(
        &mut self,
        message: UIMessage,
        connector: Arc<Mutex<ServerConnector>>,
    ) -> Task<Message>
    where
        Message: From<UIMessage> + Clone + 'a + Send + 'static,
    {
        match message {
            UIMessage::Refresh => {
                Task::future(async move { Self::refresh(connector).await }).map(|m| m.into())
            }
            UIMessage::NewQueueName(s) => {
                self.new_queue_text = s;
                Task::none()
            }
            UIMessage::CreateQueue => match self.selected_buffer_type {
                Some(queue_type) => {
                    let new_queue_name = self.new_queue_text.clone();
                    Task::perform(async move {
                        Self::create(connector.clone(), queue_type, new_queue_name).await
                    }, |_| {UIMessage::Refresh})
                    .map(|m| m.into())
                }
                None => Task::none(),
            },
            UIMessage::SelectBufferType(t) => {
                self.selected_buffer_type = Some(t);
                Task::none()
            }
            UIMessage::InspectBuffer(_) => Task::none(),
            UIMessage::NewTableData(data) => {
                match data {
                    Some(response) => {
                        self.queue_table.clear();
                        for queue_data in response {
                            self.queue_table.push(queue_data);
                        }
                    }
                    None => println!("Failed to fetch queue data."),
                }
                Task::none()
            }
        }
    }

    async fn create(
        connector: Arc<Mutex<ServerConnector>>,
        selected_buffer_type: QueueType,
        new_queue_name: String,
    ) {
        if let Ok(client) = connector.lock().await.client().await {
            if let Err(_) = client
                .transfer_admin_request(CreateQueue {
                    queue_address: QueueId::new(new_queue_name, selected_buffer_type),
                })
                .await
            {}
        }
    }

    async fn refresh(connector: Arc<Mutex<ServerConnector>>) -> UIMessage {
        UIMessage::NewTableData(match connector.lock().await.client().await {
            Ok(client) => client.transfer_admin_request(ListQueues {}).await.ok(),
            Err(err) => {
                println!("{err:?}");
                None
            },
        })
    }
}
