use crate::elements::topic_breakdown;
use crate::elements::topic_breakdown::TopicBreakdown;
use crate::server_connector::ServerConnector;
use crate::util::pretty_print_queue_dlx;
use backend::protocol::message::{Message, TTL};
use backend::protocol::queue_id::{NewQueueId, QueueFilter, QueueId, TopicLiteral};
use backend::protocol::request::{
    CreateQueue, DeleteQueue, GetSubscription, GetTopicBreakdown, Publish, Receive, Subscribe,
};
use backend::protocol::routing_key::{DLXPreference, RoutingKey};
use backend::protocol::{QueueProperties, Status, UserQueueProperties};
use iced::application::Update;
use iced::widget::{button, checkbox, column, row, slider, text, text_input, vertical_rule};
use iced::{Alignment, Element, Task};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

#[derive(Clone, Debug)]
pub enum InspectViewMessage {
    Delete(QueueId),
    Deleted,
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
    LoadBreakdown,
    BreakdownLoaded(Option<Vec<(String, Vec<String>)>>),
    CreateSubtopic(String, Option<String>),
    SubtopicCreated,
    BreakdownMessage(topic_breakdown::Message),
    GetSubscription,
    Subscription(Option<QueueFilter>),
}

pub struct InspectView {
    queue_id: QueueId,
    props: QueueProperties,
    message_body: String,
    received_message: String,
    connector: Arc<Mutex<ServerConnector>>,
    ttl_value: u16,
    ttl_permanent: bool,
    breakdown_view: TopicBreakdown,
    subscription: Option<QueueFilter>,
}

impl InspectView {
    pub fn new(
        connector: Arc<Mutex<ServerConnector>>,
        queue_id: QueueId,
        props: QueueProperties,
    ) -> (Self, Task<InspectViewMessage>) {
        let connector2 = connector.clone();
        (
            Self {
                queue_id,
                props,
                message_body: String::new(),
                received_message: String::new(),
                connector,
                ttl_value: 50,
                ttl_permanent: false,
                breakdown_view: TopicBreakdown::new("Breakdown".into()),
                subscription: None,
            },
            Task::perform(Self::get_subscription(connector2), move |payload| {
                InspectViewMessage::Subscription(payload)
            }),
        )
    }

    pub fn view(&self) -> Element<InspectViewMessage> {
        let mut delete_btn = button("Delete");
        if !self.props.system.is_system {
            delete_btn = delete_btn.on_press(InspectViewMessage::Delete(self.queue_id.clone()));
        }
        column![row![
            column![
                text("Administration").align_x(Alignment::Center),
                text(format!(
                    "DLX: {}",
                    pretty_print_queue_dlx(&self.props.user.dlx)
                )),
                text(format!("Is DLX: {}", self.props.user.is_dlx)),
                text(format!(
                    "Is System Managed: {}",
                    self.props.system.is_system
                )),
                delete_btn,
            ],
            vertical_rule(1),
            column![
                button("get breakdown").on_press(InspectViewMessage::LoadBreakdown),
                self.breakdown_view.view().map(|msg| {
                    match msg {
                        topic_breakdown::Message::CreateSubtopic(s, ss) => {
                            InspectViewMessage::CreateSubtopic(s, ss)
                        }
                        m => InspectViewMessage::BreakdownMessage(m),
                    }
                }),
                text("Messaging").align_x(Alignment::Center),
                text_input("Message body", self.message_body.as_str())
                    .on_input(InspectViewMessage::MessageBodyChanged),
                row![
                    slider(0..=300, self.ttl_value, InspectViewMessage::TTLValueChanged),
                    text(format!("{}s", self.ttl_value)),
                    checkbox("Permanent", self.ttl_permanent)
                        .on_toggle(InspectViewMessage::TTLPermanentToggle),
                    button("Send Message").on_press(InspectViewMessage::SendMessage)
                ]
                .spacing(10),
                row![
                    button("Subscribe").on_press(InspectViewMessage::Subscribe),
                    button("Receive Message").on_press(InspectViewMessage::ReceiveMessage),
                    text(self.received_message.as_str())
                ],
                text(match &self.subscription {
                    None => "Not subscribed to any queue.".to_string(),
                    Some(queue) => {
                        let is_this_queue = match (&queue, &self.queue_id) {
                            (QueueFilter::Queue(a), QueueId::Queue(b)) => a == b,
                            (QueueFilter::Topic(a, _, _), QueueId::Topic(b, _, _)) => a == b,
                            _ => false,
                        };
                        if is_this_queue {
                            queue.to_string()
                        } else {
                            "Not subscribed to this queue.".to_string()
                        }
                    }
                }),
            ]
            .spacing(10),
        ]
        .spacing(10)
        .padding(10),]
        .into()
    }

    pub fn update(&mut self, message: InspectViewMessage) -> Task<InspectViewMessage> {
        match message {
            InspectViewMessage::Delete(addr) => {
                let connector = self.connector.clone();
                let addr2 = addr.clone();
                return Task::perform(
                    async move { Self::delete_buffer(connector, addr2).await },
                    move |result| InspectViewMessage::Deleted,
                );
            }
            InspectViewMessage::MessageBodyChanged(s) => self.message_body = s,
            InspectViewMessage::SendMessage => {
                let queue = match self.queue_id.clone() {
                    QueueId::Topic(name, _, _) => match self.breakdown_view.selected_topic() {
                        Some((TopicLiteral::Name(f1), TopicLiteral::Name(f2))) => {
                            QueueId::Topic(name, f1, f2)
                        }
                        _ => {
                            println!("No topic selected, couldn't send message!");
                            return Task::none();
                        }
                    },
                    q => q,
                };
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
                );
            }
            InspectViewMessage::MessageSent => {}
            InspectViewMessage::SendFailure => {}
            InspectViewMessage::Subscribe => {
                let connector = self.connector.clone();
                let connector2 = self.connector.clone();
                let queue = match self.queue_id.clone() {
                    QueueId::Topic(name, _, _) => match self.breakdown_view.selected_topic() {
                        None => return Task::none(),
                        Some((f1, f2)) => QueueFilter::Topic(name, f1, f2),
                    },
                    q => QueueFilter::from(q),
                };
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
                .chain(Task::perform(
                    Self::get_subscription(connector2),
                    move |payload| InspectViewMessage::Subscription(payload),
                ));
            }
            InspectViewMessage::Subscribed => {}
            InspectViewMessage::ReceiveMessage => {
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
                );
            }
            InspectViewMessage::MessageReceived(m) => self.received_message = m,
            InspectViewMessage::NoMessageAvailable => self.received_message.clear(),
            InspectViewMessage::TTLValueChanged(val) => self.ttl_value = val,
            InspectViewMessage::TTLPermanentToggle(toggle) => self.ttl_permanent = toggle,
            InspectViewMessage::LoadBreakdown => {
                if let QueueId::Topic(topic_name, _, _) = &self.queue_id {
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
                    );
                }
            }
            InspectViewMessage::BreakdownLoaded(breakdown) => {
                if let Some(data) = breakdown {
                    self.breakdown_view.set_data(data)
                }
            }
            InspectViewMessage::CreateSubtopic(s, ss) => {
                if let QueueId::Topic(topic, _, _) = &self.queue_id {
                    let connector = self.connector.clone();
                    let topic = NewQueueId::Topic(topic.clone(), Some((s.clone(), ss.clone())));
                    return Task::perform(
                        async move { Self::create(connector.clone(), topic).await },
                        |_| InspectViewMessage::SubtopicCreated,
                    );
                }
            }
            InspectViewMessage::SubtopicCreated => {}
            InspectViewMessage::BreakdownMessage(msg) => self.breakdown_view.update(msg),
            InspectViewMessage::Deleted => {}
            InspectViewMessage::GetSubscription => {
                let connector = self.connector.clone();
                return Task::perform(Self::get_subscription(connector), move |payload| {
                    InspectViewMessage::Subscription(payload)
                });
            }
            InspectViewMessage::Subscription(subscription) => {
                self.subscription = subscription;
            }
        }
        Task::none()
    }

    async fn get_subscription(connector: Arc<Mutex<ServerConnector>>) -> Option<QueueFilter> {
        let mut binding = connector.lock().await;
        let client = binding.client().await.ok()?;
        client
            .transfer_admin_request(GetSubscription {})
            .await
            .ok()?
    }

    async fn create(connector: Arc<Mutex<ServerConnector>>, queue_id: NewQueueId) {
        if let Ok(client) = connector.lock().await.client().await {
            if let Err(_) = client
                .transfer_admin_request(CreateQueue {
                    queue_address: queue_id,
                    properties: UserQueueProperties {
                        is_dlx: false,
                        dlx: None,
                    },
                })
                .await
            {}
        }
    }

    async fn delete_buffer(connector: Arc<Mutex<ServerConnector>>, s: QueueId) -> Option<Status> {
        if let Ok(client) = connector.lock().await.client().await {
            client
                .transfer_admin_request(DeleteQueue { queue_name: s })
                .await
                .ok()
        } else {
            None
        }
    }
}
