use crate::elements::bool_badge::bool_badge;
use crate::elements::queue_selector;
use crate::elements::queue_selector::QueueSelector;
use crate::elements::table::Table;
use crate::elements::warning::Warning;
use crate::fonts::{font_heading, ELEMENT_SPACING_HORIZONTAL, SIZE_HEADING};
use crate::make_request::request_task;
use crate::server_connector::ServerConnector;
use crate::util::pretty_print_queue_dlx;
use backend::protocol::message::{Message, TTL};
use backend::protocol::queue_id::{NewQueueId, QueueFilter, QueueId};
use backend::protocol::request::{
    CreateQueue, DeleteQueue, GetSubscription, GetTopicBreakdown, Publish, Receive, Subscribe,
};
use backend::protocol::routing_key::{DLXPreference, RoutingKey};
use backend::protocol::{QueueProperties, UserQueueProperties};
use iced::widget::{
    button, checkbox, column, horizontal_space, row, slider, text, text_input, vertical_rule,
    Column, Row,
};
use iced::{Alignment, Element, Length, Padding, Task};
use std::collections::VecDeque;
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
    message_log: VecDeque<String>,
    queue_selector: T,
}

impl<T: QueueSelector + 'static> InspectView<T> {
    pub fn new(
        connector: Arc<Mutex<ServerConnector>>,
        queue_id: QueueId,
        props: QueueProperties,
        queue_selector: T,
    ) -> (Self, Task<Result<InspectViewMessage, ()>>) {
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
                message_log: VecDeque::new(),
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

        let mut message_log = Column::new();
        for message in &self.message_log {
            message_log = message_log.push(text(message));
        }

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
                        .push(
                            text("DLX"),
                            text(pretty_print_queue_dlx(&self.props.user.dlx))
                        )
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
                .spacing(ELEMENT_SPACING_HORIZONTAL),
                text("Receiving"),
                row![
                    button("Subscribe").on_press(InspectViewMessage::Subscribe),
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
                    })
                ]
                .spacing(ELEMENT_SPACING_HORIZONTAL),
                Row::new()
                    .push(button("Receive Message").on_press(InspectViewMessage::ReceiveMessage))
                    .push(text(format!("Received: {}", self.received_message))),
                message_log.padding(Padding::ZERO.left(20)).spacing(2)
            ]
            .spacing(ELEMENT_SPACING_HORIZONTAL),
        ]
        .spacing(ELEMENT_SPACING_HORIZONTAL)
        .padding(ELEMENT_SPACING_HORIZONTAL),]
        .into()
    }

    pub fn update(&mut self, message: InspectViewMessage) -> Task<Result<InspectViewMessage, ()>> {
        match message {
            InspectViewMessage::Delete(addr) => {
                return request_task(
                    self.connector.clone(),
                    DeleteQueue { queue_name: addr },
                    |_| InspectViewMessage::Deleted,
                );
            }
            InspectViewMessage::MessageBodyChanged(s) => self.message_body = s,
            InspectViewMessage::SendMessage(queue) => {
                let ttl = if self.ttl_permanent {
                    TTL::Permanent
                } else {
                    TTL::Duration(Duration::from_secs(self.ttl_value as u64))
                };
                return request_task(
                    self.connector.clone(),
                    Publish {
                        message: Message {
                            payload: self.message_body.clone(),
                            routing_key: RoutingKey {
                                id: queue,
                                dlx: DLXPreference::Default,
                            },
                            ttl,
                        },
                    },
                    |a| match a {
                        Ok(_) => InspectViewMessage::MessageSent,
                        Err(_) => InspectViewMessage::SendFailure,
                    },
                );
            }
            InspectViewMessage::MessageSent => {}
            InspectViewMessage::SendFailure => {}
            InspectViewMessage::Subscribe => {
                return request_task(
                    self.connector.clone(),
                    Subscribe {
                        queue: self.queue_selector.selected_filter().into(),
                    },
                    |_| InspectViewMessage::Subscribed,
                )
                .chain(Self::get_subscription_task(self.connector.clone()));
            }
            InspectViewMessage::Subscribed => {}
            InspectViewMessage::ReceiveMessage => {
                return request_task(self.connector.clone(), Receive {}, |result| match result {
                    Some(Message { payload, .. }) => InspectViewMessage::MessageReceived(payload),
                    _ => InspectViewMessage::NoMessageAvailable,
                });
            }
            InspectViewMessage::MessageReceived(m) => {
                self.message_log.push_front(self.received_message.clone());
                self.message_log.truncate(5);
                self.received_message = m;
            }
            InspectViewMessage::NoMessageAvailable => {
                self.received_message = "No message".to_string()
            }
            InspectViewMessage::TTLValueChanged(val) => self.ttl_value = val,
            InspectViewMessage::TTLPermanentToggle(toggle) => self.ttl_permanent = toggle,
            InspectViewMessage::LoadBreakdown => {
                if let QueueId::Topic(topic_name, _, _) = &self.queue_id {
                    return Self::load_breakdown_task(self.connector.clone(), topic_name.clone());
                }
            }
            InspectViewMessage::Selector(queue_selector::Message::CreateSubtopic(s, ss)) => {
                if let QueueId::Topic(topic, _, _) = &self.queue_id {
                    return request_task(
                        self.connector.clone(),
                        CreateQueue {
                            queue_address: NewQueueId::Topic(
                                topic.clone(),
                                Some((s.clone(), ss.clone())),
                            ),
                            properties: UserQueueProperties {
                                is_dlx: false,
                                dlx: None,
                            },
                        },
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

    fn load_breakdown_task(
        connector: Arc<Mutex<ServerConnector>>,
        topic_name: String,
    ) -> Task<Result<InspectViewMessage, ()>> {
        request_task(connector, GetTopicBreakdown { topic_name }, move |d| {
            InspectViewMessage::Selector(queue_selector::Message::BreakdownLoaded(d))
        })
    }

    fn get_subscription_task(
        connector: Arc<Mutex<ServerConnector>>,
    ) -> Task<Result<InspectViewMessage, ()>> {
        request_task(connector, GetSubscription {}, move |payload| {
            InspectViewMessage::Subscription(payload)
        })
    }
}
