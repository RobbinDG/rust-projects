use crate::elements::collapsible::Collapsible;
use crate::elements::connection_interface::{ConnectionInterface, ConnectionInterfaceMessage};
use crate::elements::inspect_view::{InspectView, InspectViewMessage};
use crate::elements::overlay_dialog::{Message, OverlayDialog};
use crate::elements::queue_view::UIMessage;
use crate::elements::{collapsible, QueueView};
use crate::server_connector::ServerConnector;
use backend::protocol::queue_id::QueueId;
use backend::protocol::request::GetProperties;
use backend::protocol::QueueProperties;
use iced::widget::{column, text, vertical_space};
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
    Toggled,
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

pub struct AdminView {
    connector: Arc<Mutex<ServerConnector>>,

    // Sub-widgets
    collapsible: Collapsible,
    buffer_view: QueueView,
    inspect_view: Option<OverlayDialog<InspectView>>,
    connection_interface: ConnectionInterface,
}

impl Default for AdminView {
    fn default() -> Self {
        let connector = Arc::new(Mutex::new(ServerConnector::new()));
        Self {
            collapsible: Collapsible::new("test".into(), false),
            connector: connector.clone(),
            buffer_view: QueueView::default(),
            inspect_view: None,
            connection_interface: ConnectionInterface::new(),
        }
    }
}

impl AdminView {
    pub fn view(&self) -> Element<AdminViewMessage> {
        match &self.inspect_view {
            Some(inspect_view) => inspect_view
                .view(|inspect| inspect.view())
                .map(|msg| match msg {
                    Message::Close => AdminViewMessage::CloseInspect,
                    Message::Dialog(msg) => AdminViewMessage::from(msg),
                })
                .into(),
            None => {
                let mut cols = column![
                    self.collapsible
                        .view::<()>(|| { text("collapsed").into() })
                        .map(|m| match m {
                            collapsible::Message::Toggle => AdminViewMessage::Toggled,
                            collapsible::Message::Body(_) => AdminViewMessage::Nothing,
                        }),
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
            AdminViewMessage::Inspector(m) => {
                if let Some(inspect_view) = &mut self.inspect_view {
                    inspect_view.update(|inspect| inspect.update(m)).map(AdminViewMessage::from)
                } else {
                    Task::none()
                }
            }
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
                self.inspect_view = Some(OverlayDialog::new(
                    "bla".into(),
                    InspectView::new(self.connector.clone(), address, properties),
                ));
                Task::none()
            }
            AdminViewMessage::ConnectionUpdated(m) => {
                self.connection_interface.update(m, self.connector.clone())
            }
            AdminViewMessage::Toggled => {
                self.collapsible.toggle();
                Task::none()
            }
            AdminViewMessage::CloseInspect => {
                if let Some(inspect_view) = self.inspect_view.take() {
                    inspect_view.close();
                }
                Task::none()
            }
            _ => Task::none(),
        }
    }
}
