use crate::elements::QueueTable;
use crate::server_connector::ServerConnector;
use backend::protocol::queue_id::{QueueId, QueueType};
use backend::protocol::request::{CreateQueue, ListQueues};
use backend::protocol::UserQueueProperties;
use iced::widget::{button, column, combo_box, radio, row, text_input};
use iced::{Element, Task};
use std::fmt::{Display, Formatter};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Debug)]
struct DLXChoice {
    value: Option<QueueId>,
}

#[derive(Debug, Clone)]
pub enum UIMessage {
    Refresh,
    NewTableData(Option<Vec<(QueueId, usize, usize, usize)>>),
    NewQueueName(String),
    CreateQueue,
    SelectBufferType(QueueType),
    InspectBuffer(QueueId),
    SetDLX(DLXChoice),
}

impl Display for DLXChoice {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.value {
            None => write!(f, "System Default"),
            Some(id) => write!(f, "{}", id.to_string()),
        }
    }
}

pub struct QueueView {
    // Widget state
    queue_table: QueueTable,
    new_queue_text: String,
    selected_buffer_type: Option<QueueType>,
    dlx_state: combo_box::State<DLXChoice>,
    current_dlx: DLXChoice,
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
            dlx_state: combo_box::State::new(vec![]),
            current_dlx: DLXChoice { value: None },
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
                combo_box(
                    &self.dlx_state,
                    "Choose DLX",
                    Some(&self.current_dlx),
                    UIMessage::SetDLX
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

    pub fn update<Message>(
        &mut self,
        message: UIMessage,
        connector: Arc<Mutex<ServerConnector>>,
    ) -> Task<Message>
    where
        Message: From<UIMessage> + Clone + Send + 'static,
    {
        match message {
            UIMessage::Refresh => {
                return Task::future(async move { Self::refresh(connector).await })
                    .map(|m| m.into())
            }
            UIMessage::NewQueueName(s) => {
                self.new_queue_text = s;
            }
            UIMessage::CreateQueue => match self.selected_buffer_type {
                Some(queue_type) => {
                    let new_queue_name = self.new_queue_text.clone();
                    let dlx_choice = self.current_dlx.clone();
                    return Task::perform(async move {
                        Self::create(connector.clone(), queue_type, new_queue_name, dlx_choice).await
                    }, |_| { UIMessage::Refresh })
                        .map(|m| m.into());
                }
                None => {}
            },
            UIMessage::SelectBufferType(t) => {
                self.selected_buffer_type = Some(t);
            }
            UIMessage::InspectBuffer(_) => {}
            UIMessage::NewTableData(data) => match data {
                Some(response) => {
                    let mut options = vec![DLXChoice { value: None }];
                    self.queue_table.clear();
                    for queue_data in response {
                        options.push(DLXChoice {
                            value: Some(queue_data.0.clone()),
                        });
                        self.queue_table.push(queue_data);
                    }
                    self.dlx_state = combo_box::State::new(options);
                }
                None => println!("Failed to fetch queue data."),
            },
            UIMessage::SetDLX(choice) => self.current_dlx = choice,
        }
        Task::none()
    }

    async fn create(
        connector: Arc<Mutex<ServerConnector>>,
        selected_buffer_type: QueueType,
        new_queue_name: String,
        dlx: DLXChoice
    ) {
        if let Ok(client) = connector.lock().await.client().await {
            if let Err(_) = client
                .transfer_admin_request(CreateQueue {
                    queue_address: QueueId::new(new_queue_name, selected_buffer_type),
                    properties: UserQueueProperties {
                        is_dlx: false, // TODO create UI element to configure
                        dlx: dlx.value,
                    },
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
            }
        })
    }
}
