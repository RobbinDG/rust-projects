use crate::elements::collapsible;
use crate::elements::collapsible::Collapsible;
use crate::server_connector::ServerConnector;
use crate::util::pretty_print_queue_dlx;
use backend::protocol::message::{Message, TTL};
use backend::protocol::queue_id::QueueId;
use backend::protocol::request::{GetTopicBreakdown, Publish, Receive, Subscribe};
use backend::protocol::routing_key::{DLXPreference, RoutingKey};
use backend::protocol::QueueProperties;
use iced::widget::{
    button, checkbox, column, horizontal_rule, row, slider, text, text_input, vertical_rule,
};
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
    Subscribe,
    Subscribed,
    ReceiveMessage,
    MessageReceived(String),
    NoMessageAvailable,
    TTLValueChanged(u16),
    TTLPermanentToggle(bool),
    ToggleBreakdown(Option<usize>),
    LoadBreakdown,
    BreakdownLoaded(Option<Vec<(String, Vec<String>)>>),
}

pub struct InspectView {
    pub buffer_info: Option<(QueueId, QueueProperties)>,
    message_body: String,
    received_message: String,
    connector: Arc<Mutex<ServerConnector>>,
    ttl_value: u16,
    ttl_permanent: bool,
    breakdown_view: Collapsible,
    sub_breakdown_views: Vec<(Collapsible, Vec<String>)>,
}

impl InspectView {
    pub fn new(connector: Arc<Mutex<ServerConnector>>) -> Self {
        Self {
            buffer_info: None,
            message_body: String::new(),
            received_message: String::new(),
            connector,
            ttl_value: 50,
            ttl_permanent: false,
            breakdown_view: Collapsible::new("Breakdown".into(), false),
            sub_breakdown_views: Vec::new(),
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
                            text(format!(
                                "DLX: {}",
                                pretty_print_queue_dlx(&properties.user.dlx)
                            )),
                            text(format!("Is DLX: {}", properties.user.is_dlx)),
                            text(format!(
                                "Is System Managed: {}",
                                properties.system.is_system
                            )),
                            delete_btn,
                        ],
                        vertical_rule(1),
                        column![
                            button("get breakdown").on_press(InspectViewMessage::LoadBreakdown),
                            self.breakdown_view
                                .view(|| {
                                    let mut rowabc = row![];
                                    let mut e = self.sub_breakdown_views.iter().enumerate();
                                    while let Some((i, (c, s))) = e.next() {
                                        rowabc =
                                            rowabc.push(
                                                c.view::<()>(|| {
                                                    let mut row = row![];
                                                    for ss in s {
                                                        row = row.push(text(ss));
                                                    }
                                                    row.into()
                                                })
                                                .map(move |_| {
                                                    InspectViewMessage::ToggleBreakdown(Some(i))
                                                }),
                                            );
                                    }
                                    rowabc.into()
                                })
                                .map(|msg| {
                                    match msg {
                                        collapsible::Message::Toggle => {
                                            InspectViewMessage::ToggleBreakdown(None)
                                        }
                                        collapsible::Message::Body(msg) => msg,
                                    }
                                }),
                            text("Messaging").align_x(Alignment::Center),
                            text_input("Message body", self.message_body.as_str())
                                .on_input(InspectViewMessage::MessageBodyChanged),
                            row![
                                slider(
                                    0..=300,
                                    self.ttl_value,
                                    InspectViewMessage::TTLValueChanged
                                ),
                                text(format!("{}s", self.ttl_value)),
                                checkbox("Permanent", self.ttl_permanent)
                                    .on_toggle(InspectViewMessage::TTLPermanentToggle),
                                button("Send Message").on_press(InspectViewMessage::SendMessage)
                            ]
                            .spacing(10),
                            row![
                                button("Subscribe").on_press(InspectViewMessage::Subscribe),
                                button("Receive Message")
                                    .on_press(InspectViewMessage::ReceiveMessage),
                                text(self.received_message.as_str())
                            ],
                        ]
                        .spacing(10),
                    ]
                    .spacing(10)
                    .padding(10),
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
                    let ttl = if self.ttl_permanent {
                        TTL::Permanent
                    } else {
                        TTL::Duration(Duration::from_secs(self.ttl_value as u64))
                    };
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
                                        ttl,
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
            InspectViewMessage::Subscribe => {
                if let Some((queue, _)) = &self.buffer_info {
                    let connector = self.connector.clone();
                    let queue = queue.clone();
                    return Task::perform(
                        async move {
                            let mut binding = connector.lock().await;
                            let client = binding.client().await.ok()?;
                            client
                                .transfer_admin_request(Subscribe {
                                    queue: queue.into(),
                                })
                                .await
                                .ok()
                        },
                        move |result| InspectViewMessage::Subscribed,
                    )
                    .map(Msg::from);
                }
            }
            InspectViewMessage::Subscribed => {}
            InspectViewMessage::ReceiveMessage => {
                if self.buffer_info.is_some() {
                    let connector = self.connector.clone();
                    return Task::perform(
                        async move {
                            let mut binding = connector.lock().await;
                            let client = binding.client().await.ok()?;
                            client.transfer_admin_request(Receive {}).await.ok()?
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
            InspectViewMessage::TTLValueChanged(val) => self.ttl_value = val,
            InspectViewMessage::TTLPermanentToggle(toggle) => self.ttl_permanent = toggle,
            InspectViewMessage::ToggleBreakdown(menu) => match menu {
                None => self.breakdown_view.toggle(),
                Some(i) => {
                    if let Some((c, _)) = self.sub_breakdown_views.get_mut(i) {
                        c.toggle();
                    }
                }
            },
            InspectViewMessage::LoadBreakdown => {
                if let Some((QueueId::Topic(topic_name, _, _), _)) = &self.buffer_info {
                    let connector = self.connector.clone();
                    let topic_name = topic_name.clone();
                    return Task::perform(
                        async move {
                            let mut binding = connector.lock().await;
                            let client = binding.client().await.ok()?;
                            client
                                .transfer_admin_request(GetTopicBreakdown { topic_name })
                                .await
                                .ok()?
                        },
                        move |payload| InspectViewMessage::BreakdownLoaded(payload),
                    )
                    .map(Msg::from);
                }
            }
            InspectViewMessage::BreakdownLoaded(breakdown) => {
                self.sub_breakdown_views.clear();
                if let Some(breakdown) = breakdown {
                    for (s, ss) in breakdown {
                        self.sub_breakdown_views
                            .push((Collapsible::new(s, false), ss))
                    }
                }
            }
        }
        Task::none()
    }
}
