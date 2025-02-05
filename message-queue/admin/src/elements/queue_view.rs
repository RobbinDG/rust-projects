use crate::elements::QueueTable;
use crate::make_request::request_task;
use crate::server_connector::ServerConnector;
use crate::util::pretty_print_queue_dlx;
use backend::protocol::queue_id::{NewQueueId, QueueId, QueueType, TopLevelQueueId};
use backend::protocol::request::{CreateQueue, ListQueues};
use backend::protocol::UserQueueProperties;
use iced::widget::{button, checkbox, column, combo_box, radio, row, text_input};
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
    NewTableData(Vec<(TopLevelQueueId, usize, usize)>),
    NewQueueName(String),
    CreateQueue,
    SelectBufferType(QueueType),
    InspectBuffer(TopLevelQueueId),
    SetDLXChoice(DLXChoice),
    SetIsDLX(bool),
}

impl Display for DLXChoice {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", pretty_print_queue_dlx(&self.value))
    }
}

pub struct QueueView {
    // Widget state
    queue_table: QueueTable,
    new_queue_text: String,
    selected_buffer_type: Option<QueueType>,
    dlx_state: combo_box::State<DLXChoice>,
    current_dlx: DLXChoice,
    is_dlx: bool,
}

impl Default for QueueView {
    fn default() -> Self {
        QueueView {
            queue_table: QueueTable::new(["Queue", "Subscribers", "Messages"], [300, 200, 200]),
            new_queue_text: String::new(),
            selected_buffer_type: Some(QueueType::Queue),
            dlx_state: combo_box::State::new(vec![]),
            current_dlx: DLXChoice { value: None },
            is_dlx: false,
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
                    UIMessage::SetDLXChoice
                ),
                checkbox("Is DLX", self.is_dlx).on_toggle(UIMessage::SetIsDLX),
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

    pub fn update(
        &mut self,
        message: UIMessage,
        connector: Arc<Mutex<ServerConnector>>,
    ) -> Task<Result<UIMessage, ()>> {
        match message {
            UIMessage::Refresh => {
                return request_task(connector.clone(), ListQueues {}, UIMessage::NewTableData);
            }
            UIMessage::NewQueueName(s) => {
                self.new_queue_text = s;
            }
            UIMessage::CreateQueue => match self.selected_buffer_type {
                Some(queue_type) => {
                    return request_task(
                        connector.clone(),
                        CreateQueue {
                            queue_address: match queue_type {
                                QueueType::Queue => NewQueueId::Queue(self.new_queue_text.clone()),
                                QueueType::Topic => {
                                    NewQueueId::Topic(self.new_queue_text.clone(), None)
                                }
                            },
                            properties: UserQueueProperties {
                                is_dlx: self.is_dlx,
                                dlx: self.current_dlx.value.clone(),
                            },
                        },
                        |_| UIMessage::Refresh,
                    );
                }
                None => {}
            },
            UIMessage::SelectBufferType(t) => {
                self.selected_buffer_type = Some(t);
            }
            UIMessage::InspectBuffer(_) => {}
            UIMessage::NewTableData(data) => {
                let mut options = vec![DLXChoice { value: None }];
                self.queue_table.clear();
                for queue_data in data {
                    if let TopLevelQueueId::Queue(name) = &queue_data.0 {
                        options.push(DLXChoice {
                            value: Some(QueueId::Queue(name.clone())),
                        });
                    }
                    self.queue_table.push(queue_data);
                }
                self.dlx_state = combo_box::State::new(options);
            }
            UIMessage::SetDLXChoice(choice) => self.current_dlx = choice,
            UIMessage::SetIsDLX(toggle) => self.is_dlx = toggle,
        }
        Task::none()
    }
}
