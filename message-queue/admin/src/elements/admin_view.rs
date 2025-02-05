use crate::elements::connection_interface::{ConnectionInterface, ConnectionInterfaceMessage};
use crate::elements::direct_selector::DirectSelector;
use crate::elements::inspect_view::{InspectView, InspectViewMessage};
use crate::elements::overlay_dialog::OverlayDialog;
use crate::elements::queue_view::UIMessage;
use crate::elements::topic_selector::TopicSelector;
use crate::elements::{overlay_dialog, QueueView};
use crate::make_request::request_task;
use crate::server_connector::ServerConnector;
use backend::protocol::queue_id::QueueId;
use backend::protocol::request::GetProperties;
use backend::protocol::QueueProperties;
use iced::widget::{column, vertical_space};
use iced::{Element, Task};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Debug)]
pub enum Message {
    View(AdminViewMessage),
    RequestSuccess(AdminViewMessage),
}

#[derive(Clone, Debug)]
pub enum AdminViewMessage {
    InspectBuffer(QueueId),
    InspectInfo(QueueId, QueueProperties),
    CloseInspect,
    BufferView(UIMessage),
    Inspector(InspectViewMessage),
    ConnectionUpdated(ConnectionInterfaceMessage),
}

impl From<ConnectionInterfaceMessage> for AdminViewMessage {
    fn from(value: ConnectionInterfaceMessage) -> Self {
        AdminViewMessage::ConnectionUpdated(value)
    }
}

impl From<UIMessage> for AdminViewMessage {
    fn from(msg: UIMessage) -> Self {
        AdminViewMessage::BufferView(msg)
    }
}

impl From<InspectViewMessage> for AdminViewMessage {
    fn from(msg: InspectViewMessage) -> Self {
        AdminViewMessage::Inspector(msg)
    }
}

enum Inspect {
    None,
    Direct(OverlayDialog<InspectView<DirectSelector>>),
    Topic(OverlayDialog<InspectView<TopicSelector>>),
}

impl Inspect {
    pub fn take(&mut self) -> Self {
        std::mem::replace(self, Self::None)
    }
}

pub struct AdminView {
    connector: Arc<Mutex<ServerConnector>>,

    // Sub-widgets
    buffer_view: QueueView,
    inspect_view: Inspect,
    connection_interface: ConnectionInterface,
}

impl Default for AdminView {
    fn default() -> Self {
        let address = "127.0.0.1:1234".to_string();
        let connector = Arc::new(Mutex::new(ServerConnector::new(address.clone())));
        Self {
            connector,
            buffer_view: QueueView::default(),
            inspect_view: Inspect::None,
            connection_interface: ConnectionInterface::new(address),
        }
    }
}

impl AdminView {
    pub fn view(&self) -> Element<Message> {
        let element: Element<AdminViewMessage> = match &self.inspect_view {
            Inspect::Topic(inspect_view) => inspect_view
                .view(|inspect| inspect.view())
                .map(|msg| match msg {
                    overlay_dialog::Message::Close => AdminViewMessage::CloseInspect,
                    overlay_dialog::Message::Dialog(msg) => AdminViewMessage::from(msg),
                })
                .into(),
            Inspect::Direct(inspect_view) => inspect_view
                .view(|inspect| inspect.view())
                .map(|msg| match msg {
                    overlay_dialog::Message::Close => AdminViewMessage::CloseInspect,
                    overlay_dialog::Message::Dialog(msg) => AdminViewMessage::from(msg),
                })
                .into(),
            Inspect::None => {
                let mut cols = column![
                    self.buffer_view.view().map(|message| match message {
                        UIMessage::InspectBuffer(t) => AdminViewMessage::InspectBuffer(t),
                        message => message.into(),
                    }),
                    vertical_space(),
                    self.connection_interface.view(),
                ];

                cols = cols.spacing(2).padding(10);
                cols.into()
            }
        };
        element.map(Message::View)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::View(msg) => self.update_view_message(msg),
            Message::RequestSuccess(msg) => {
                self.connection_interface.set_connected(true);
                self.update_view_message(msg)
            }
        }
    }

    pub fn update_view_message(&mut self, message: AdminViewMessage) -> Task<Message> {
        match message {
            AdminViewMessage::BufferView(m) => {
                return self.buffer_view.update(m, self.connector.clone()).map(
                    |result| match result {
                        Ok(val) => Message::RequestSuccess(AdminViewMessage::from(val)),
                        Err(_) => Message::View(AdminViewMessage::ConnectionUpdated(
                            ConnectionInterfaceMessage::Connected(false),
                        )),
                    },
                )
            }
            AdminViewMessage::Inspector(m) => {
                return match &mut self.inspect_view {
                    Inspect::None => Task::none(),
                    Inspect::Direct(inspect_view) => {
                        inspect_view.update(|inspect| inspect.update(m))
                    }
                    Inspect::Topic(inspect_view) => {
                        inspect_view.update(|inspect| inspect.update(m))
                    }
                }
                .map(Self::map_task)
            }
            AdminViewMessage::InspectBuffer(address) => {
                return request_task(
                    self.connector.clone(),
                    GetProperties {
                        queue: address.clone(),
                    },
                    move |response| {
                        AdminViewMessage::InspectInfo(address.clone(), response.unwrap())
                    },
                )
                .map(|result| match result {
                    Ok(r) => Message::RequestSuccess(r),
                    Err(_) => Message::View(AdminViewMessage::ConnectionUpdated(
                        ConnectionInterfaceMessage::Connected(false),
                    )),
                })
            }
            AdminViewMessage::InspectInfo(address, properties) => {
                let (view, load_task) = match &address {
                    QueueId::Queue(name) => {
                        let title = address.to_string();
                        let name = name.clone();
                        let (inspect_view, load_task) = InspectView::new(
                            self.connector.clone(),
                            address,
                            properties,
                            DirectSelector::new(name),
                        );
                        (
                            Inspect::Direct(OverlayDialog::new(title, inspect_view)),
                            load_task,
                        )
                    }
                    QueueId::Topic(name, _, _) => {
                        let title = address.to_string();
                        let name = name.clone();
                        let (inspect_view, load_task) = InspectView::new(
                            self.connector.clone(),
                            address,
                            properties,
                            TopicSelector::new(name),
                        );
                        (
                            Inspect::Topic(OverlayDialog::new(title, inspect_view)),
                            load_task,
                        )
                    }
                };
                self.inspect_view = view;
                return load_task.map(Self::map_task);
            }
            AdminViewMessage::ConnectionUpdated(m) => self
                .connection_interface
                .update(m, self.connector.clone())
                .map(AdminViewMessage::ConnectionUpdated),
            AdminViewMessage::CloseInspect => {
                match self.inspect_view.take() {
                    Inspect::Direct(inspect_view) => {
                        inspect_view.close();
                    }
                    Inspect::Topic(inspect_view) => {
                        inspect_view.close();
                    }
                    _ => {}
                }
                self.inspect_view = Inspect::None;
                Task::none()
            }
        }
        .map(Message::View)
    }

    pub fn map_task<M>(msg: Result<M, ()>) -> Message
    where
        AdminViewMessage: From<M>,
    {
        match msg {
            Ok(msg) => Message::RequestSuccess(AdminViewMessage::from(msg)),
            Err(_) => Message::View(AdminViewMessage::ConnectionUpdated(
                ConnectionInterfaceMessage::Connected(false),
            )),
        }
    }
}
