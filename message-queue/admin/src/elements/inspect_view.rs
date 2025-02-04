use crate::elements::bool_badge::bool_badge;
use crate::elements::queue_selector;
use crate::elements::queue_selector::QueueSelector;
use crate::elements::table::Table;
use crate::elements::warning::Warning;
use crate::fonts::{font_heading, ELEMENT_SPACING, SIZE_HEADING};
use crate::server_connector::ServerConnector;
use crate::util::pretty_print_queue_dlx;
use backend::protocol::message::{Message, TTL};
use backend::protocol::queue_id::{NewQueueId, QueueFilter, QueueId};
use backend::protocol::request::{
    CreateQueue, DeleteQueue, GetSubscription, GetTopicBreakdown, Publish, Receive, Subscribe,
};
use backend::protocol::routing_key::{DLXPreference, RoutingKey};
use backend::protocol::{QueueProperties, Status, UserQueueProperties};
use iced::widget::{
    button, checkbox, column, horizontal_space, row, slider, text, text_input, vertical_rule,
    Column,
};
use iced::{Alignment, Element, Length, Task};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

#[derive(Clone, Debug)]
pub enum InspectViewMessage {
    Delete(QueueId),
    Deleted,
    MessageBodyChanged(String),
    SendMessage(QueueId),
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
    SubtopicCreated,
    Subscription(Option<QueueFilter>),
    Selector(queue_selector::Message),
}

pub struct InspectView<T>
where
    T: QueueSelector,
{
    queue_id: QueueId,
    props: QueueProperties,
    message_body: String,
    received_message: String,
    connector: Arc<Mutex<ServerConnector>>,
    ttl_value: u16,
    ttl_permanent: bool,
    subscription: Option<QueueFilter>,
    queue_selector: T,
}

impl<T: QueueSelector + 'static> InspectView<T> {
    pub fn new(
        connector: Arc<Mutex<ServerConnector>>,
        queue_id: QueueId,
        props: QueueProperties,
        queue_selector: T,
    ) -> (Self, Task<InspectViewMessage>) {
        let mut window_load_task = Self::get_subscription_task(connector.clone());
        if let QueueId::Topic(name, _, _) = &queue_id {
            window_load_task =
                window_load_task.chain(Self::load_breakdown_task(connector.clone(), name.clone()));
        }
        (
            Self {
                queue_id,
                props,
                message_body: String::new(),
                received_message: String::new(),
                connector,
                ttl_value: 50,
                ttl_permanent: false,
                subscription: None,
                queue_selector,
            },
            window_load_task,
        )
    }

    pub fn view(&self) -> Element<InspectViewMessage> {
        let mut delete_btn = button("Delete");
        if !self.props.system.is_system {
            delete_btn = delete_btn.on_press(InspectViewMessage::Delete(self.queue_id.clone()));
        }
        let send_queue = self.queue_selector.selected();
        let send_message_btn = match send_queue {
            Some(queue) => {
                button("Send Message").on_press(InspectViewMessage::SendMessage(queue))
            }
            None => {
                button(row![text("Send Message"), Warning::new("Cannot send a message to a topic containing a filter. Specify a concrete topic to send.")].spacing(4))
            }
        };
        column![row![
            Column::new()
                .push(
                    text("Administration")
                        .align_x(Alignment::Center)
                        .font(font_heading())
                        .size(SIZE_HEADING)
                )
                .push(
                    Table::new()
                        .push(text("DLX"), text(pretty_print_queue_dlx(&self.props.user.dlx)))
                        .push(text("Is DLX"), bool_badge(self.props.user.is_dlx))
                        .push(
                            text("Is System Managed"),
                            bool_badge(self.props.system.is_system)
                        )
                )
                .push(delete_btn),
            vertical_rule(1),
            column![
                row![
                    text("Interfacing").font(font_heading()).size(SIZE_HEADING),
                    horizontal_space().width(Length::Fill),
                    button("Reload").on_press(InspectViewMessage::LoadBreakdown),
                ],
                self.queue_selector
                    .view()
                    .map(|m| InspectViewMessage::Selector(m)),
                text("Publishing"),
                text_input("Message body", self.message_body.as_str())
                    .on_input(InspectViewMessage::MessageBodyChanged),
                row![
                    slider(0..=300, self.ttl_value, InspectViewMessage::TTLValueChanged),
                    text(format!("{}s", self.ttl_value)),
                    checkbox("Permanent", self.ttl_permanent)
                        .on_toggle(InspectViewMessage::TTLPermanentToggle),
                    send_message_btn,
                ]
                .spacing(ELEMENT_SPACING),
                text("Receiving"),
                row![
                    button("Subscribe").on_press(InspectViewMessage::Subscribe),
                    button("Receive Message").on_press(InspectViewMessage::ReceiveMessage),
                    text(self.received_message.as_str())
                ]
                .spacing(ELEMENT_SPACING),
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
            .spacing(ELEMENT_SPACING),
        ]
        .spacing(ELEMENT_SPACING)
        .padding(ELEMENT_SPACING),]
        .into()
    }

    pub fn update(&mut self, message: InspectViewMessage) -> Task<InspectViewMessage> {
        match message {
            InspectViewMessage::Delete(addr) => {
                let connector = self.connector.clone();
                let addr2 = addr.clone();
                return Task::perform(
                    async move { Self::delete_buffer(connector, addr2).await },
                    move |_| InspectViewMessage::Deleted,
                );
            }
            InspectViewMessage::MessageBodyChanged(s) => self.message_body = s,
            InspectViewMessage::SendMessage(queue) => {
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
                return self
                    .subscribe_task(self.connector.clone())
                    .chain(Self::get_subscription_task(self.connector.clone()));
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
                    return Self::load_breakdown_task(self.connector.clone(), topic_name.clone());
                }
            }
            InspectViewMessage::Selector(queue_selector::Message::CreateSubtopic(s, ss)) => {
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
            InspectViewMessage::Deleted => {}
            InspectViewMessage::Subscription(subscription) => {
                self.subscription = subscription;
            }
            InspectViewMessage::Selector(m) => self.queue_selector.update(m),
        }
        Task::none()
    }

    fn subscribe_task(
        &mut self,
        connector: Arc<Mutex<ServerConnector>>,
    ) -> Task<InspectViewMessage> {
        let queue = self.queue_selector.selected_filter();
        Task::perform(
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
            move |_| InspectViewMessage::Subscribed,
        )
    }

    fn load_breakdown_task(
        connector: Arc<Mutex<ServerConnector>>,
        topic_name: String,
    ) -> Task<InspectViewMessage> {
        Task::perform(
            async move {
                let mut binding = connector.lock().await;
                let client = binding.client().await.ok()?;
                client
                    .transfer_admin_request(GetTopicBreakdown { topic_name })
                    .await
                    .ok()?
            },
            move |d| InspectViewMessage::Selector(queue_selector::Message::BreakdownLoaded(d)),
        )
    }

    fn get_subscription_task(connector: Arc<Mutex<ServerConnector>>) -> Task<InspectViewMessage> {
        Task::perform(Self::get_subscription(connector), move |payload| {
            InspectViewMessage::Subscription(payload)
        })
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
