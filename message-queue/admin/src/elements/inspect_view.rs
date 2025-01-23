use crate::server_connector::ServerConnector;
use backend::protocol::message::{Message, TTL};
use backend::protocol::queue_id::QueueId;
use backend::protocol::request::{Publish, Receive};
use backend::protocol::routing_key::{DLXPreference, RoutingKey};
use backend::protocol::QueueProperties;
use iced::widget::{button, column, horizontal_rule, row, text, text_input, vertical_rule};
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
    ReceiveMessage,
    MessageReceived(String),
    NoMessageAvailable,
}

pub struct InspectView {
    pub buffer_info: Option<(QueueId, QueueProperties)>,
    message_body: String,
    received_message: String,
    connector: Arc<Mutex<ServerConnector>>,
}

impl InspectView {
    pub fn new(connector: Arc<Mutex<ServerConnector>>) -> Self {
        Self {
            buffer_info: None,
            message_body: String::new(),
            received_message: String::new(),
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
                if !properties.system.is_system {
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
                    horizontal_rule(1),
                    row![
                        column![
                            text("Administration").align_x(Alignment::Center),
                            delete_btn,
                        ],
                        vertical_rule(1),
                        column![
                            text("Messaging").align_x(Alignment::Center),
                            row![
                                text_input("Message body", self.message_body.as_str())
                                    .on_input(InspectViewMessage::MessageBodyChanged),
                                button("Send Message").on_press(InspectViewMessage::SendMessage)
                            ].spacing(10),
                            row![
                                button("Receive Message")
                                    .on_press(InspectViewMessage::ReceiveMessage),
                                text(self.received_message.as_str())
                            ],
                        ].spacing(10),
                    ].spacing(10).padding(10),
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
                            binding
                                .client()
                                .await
                                .ok()?
                                .transfer_admin_request(Publish {
                                    message: Message {
                                        payload: body,
                                        routing_key: RoutingKey {
                                            id: queue,
                                            dlx: DLXPreference::Default,
                                        },
                                        ttl: TTL::Duration(Duration::from_secs(5)),
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
                    )
                    .map(Msg::from);
                }
            }
            InspectViewMessage::MessageSent => {}
            InspectViewMessage::SendFailure => {}
            InspectViewMessage::ReceiveMessage => {
                if let Some((queue, _)) = &self.buffer_info {
                    let queue = queue.clone();
                    let connector = self.connector.clone();
                    return Task::perform(
                        async move {
                            let mut binding = connector.lock().await;
                            let client = binding.client().await.ok()?;
                            client
                                .transfer_admin_request(Receive { queue })
                                .await
                                .ok()?
                        },
                        move |result| match result {
                            Some(Message { payload, .. }) => {
                                InspectViewMessage::MessageReceived(payload)
                            }
                            _ => InspectViewMessage::NoMessageAvailable,
                        },
                    )
                    .map(Msg::from);
                }
            }
            InspectViewMessage::MessageReceived(m) => self.received_message = m,
            InspectViewMessage::NoMessageAvailable => self.received_message.clear(),
        }
        Task::none()
    }
}
