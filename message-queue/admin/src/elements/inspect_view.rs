use crate::server_connector::ServerConnector;
use backend::protocol::new::message::Message;
use backend::protocol::new::queue_id::QueueId;
use backend::protocol::new::routing_key::{DLXPreference, RoutingKey};
use backend::protocol::request::Publish;
use backend::protocol::BufferProperties;
use iced::widget::{button, column, row, text, text_input};
use iced::{Alignment, Element, Length, Task};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

#[derive(Clone, Debug)]
pub enum InspectViewMessage {
    Close,
    Delete,
    MessageBodyChanged(String),
    SendMessage,
    MessageSent,
    SendFailure,
}

pub struct InspectView {
    pub buffer_info: Option<(QueueId, BufferProperties)>,
    message_body: String,
    connector: Arc<Mutex<ServerConnector>>,
}

impl InspectView {
    pub fn new(connector: Arc<Mutex<ServerConnector>>) -> Self {
        Self {
            buffer_info: None,
            message_body: String::new(),
            connector,
        }
    }

    pub fn view<'a, Message>(&'a self) -> Element<'a, Message>
    where
        Message: From<InspectViewMessage> + 'a,
    {
        match &self.buffer_info {
            Some((address, properties)) => {
                let mut delete_btn = button("Delete");
                if !properties.system_buffer {
                    delete_btn = delete_btn.on_press(InspectViewMessage::Delete);
                }
                let element: Element<InspectViewMessage> = column![
                    row![
                        button("<").on_press(InspectViewMessage::Close),
                        text(format![
                            "{:?} {}",
                            address.queue_type(),
                            address.to_string()
                        ])
                        .width(Length::Fill)
                        .align_x(Alignment::Center),
                    ],
                    row![
                        text_input("Message body", self.message_body.as_str())
                            .on_input(InspectViewMessage::MessageBodyChanged),
                        button("Send Message").on_press(InspectViewMessage::SendMessage)
                    ],
                    delete_btn,
                ]
                .into();
                element.map(Message::from)
            }
            None => text("No buffer selected.").into(),
        }
    }

    pub fn update<Msg>(&mut self, message: InspectViewMessage) -> Task<Msg>
    where
        Msg: From<InspectViewMessage> + Send + 'static,
    {
        match message {
            InspectViewMessage::Delete => self.buffer_info = None,
            InspectViewMessage::Close => self.buffer_info = None,
            InspectViewMessage::MessageBodyChanged(s) => self.message_body = s,
            InspectViewMessage::SendMessage => {
                if let Some((queue, _)) = &self.buffer_info {
                    let queue = queue.clone();
                    let connector = self.connector.clone();
                    let body = self.message_body.clone();
                    return Task::perform(
                        async move {
                            let mut binding = connector.lock().await;
                            let client = binding.client().await.ok()?;
                            client
                                .transfer_admin_request(Publish {
                                    message: Message {
                                        payload: body,
                                        routing_key: RoutingKey {
                                            id: queue,
                                            dlx: DLXPreference::Default,
                                        },
                                        ttl: Duration::from_secs(100),
                                    },
                                })
                                .await
                                .ok()
                        },
                        move |result| {
                            if result.is_some() {
                                InspectViewMessage::MessageSent
                            } else {
                                InspectViewMessage::SendFailure
                            }
                        },
                    ).map(Msg::from);
                }
            }
            InspectViewMessage::MessageSent => {}
            InspectViewMessage::SendFailure => {}
        }
        Task::none()
    }
}
