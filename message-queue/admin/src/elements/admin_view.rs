use crate::elements::connection_interface::{ConnectionInterface, ConnectionInterfaceMessage};
use crate::elements::direct_selector::DirectSelector;
use crate::elements::inspect_view::{InspectView, InspectViewMessage};
use crate::elements::overlay_dialog::{Message, OverlayDialog};
use crate::elements::queue_view::UIMessage;
use crate::elements::topic_selector::TopicSelector;
use crate::elements::QueueView;
use crate::server_connector::ServerConnector;
use backend::protocol::queue_id::QueueId;
use backend::protocol::request::GetProperties;
use backend::protocol::QueueProperties;
use iced::widget::{column, vertical_space};
use iced::{Element, Task};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Debug)]
pub enum AdminViewMessage {
    InspectBuffer(QueueId),
    InspectInfo(QueueId, QueueProperties),
    CloseInspect,
    BufferView(UIMessage),
    Inspector(InspectViewMessage),
    ConnectionUpdated(ConnectionInterfaceMessage),
    Nothing,
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
        let connector = Arc::new(Mutex::new(ServerConnector::new()));
        Self {
            connector: connector.clone(),
            buffer_view: QueueView::default(),
            inspect_view: Inspect::None,
            connection_interface: ConnectionInterface::new(),
        }
    }
}

impl AdminView {
    pub fn view(&self) -> Element<AdminViewMessage> {
        match &self.inspect_view {
            Inspect::Topic(inspect_view) => inspect_view
                .view(|inspect| inspect.view())
                .map(|msg| match msg {
                    Message::Close => AdminViewMessage::CloseInspect,
                    Message::Dialog(msg) => AdminViewMessage::from(msg),
                })
                .into(),
            Inspect::Direct(inspect_view) => inspect_view
                .view(|inspect| inspect.view())
                .map(|msg| match msg {
                    Message::Close => AdminViewMessage::CloseInspect,
                    Message::Dialog(msg) => AdminViewMessage::from(msg),
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
        }
    }

    pub fn update(&mut self, message: AdminViewMessage) -> Task<AdminViewMessage> {
        match message {
            AdminViewMessage::BufferView(m) => self.buffer_view.update(m, self.connector.clone()),
            AdminViewMessage::Inspector(m) => match &mut self.inspect_view {
                Inspect::None => Task::none(),
                Inspect::Direct(inspect_view) => inspect_view.update(|inspect| inspect.update(m)),
                Inspect::Topic(inspect_view) => inspect_view.update(|inspect| inspect.update(m)),
            }
            .map(AdminViewMessage::from),
            AdminViewMessage::InspectBuffer(address) => {
                let connection = self.connector.clone();

                Task::perform(
                    async move {
                        if let Ok(client) = connection.lock().await.client().await {
                            let address_2 = address.clone();
                            Some((
                                address_2,
                                client
                                    .transfer_admin_request(GetProperties { queue: address })
                                    .await,
                            ))
                        } else {
                            None
                        }
                    },
                    move |maybe_data| match maybe_data {
                        Some((address, properties)) => {
                            AdminViewMessage::InspectInfo(address, properties.unwrap().unwrap())
                        }
                        None => AdminViewMessage::ConnectionUpdated(
                            ConnectionInterfaceMessage::Connected(false),
                        ),
                    },
                )
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
                load_task.map(AdminViewMessage::from)
            }
            AdminViewMessage::ConnectionUpdated(m) => {
                self.connection_interface.update(m, self.connector.clone())
            }
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
            _ => Task::none(),
        }
    }
}
