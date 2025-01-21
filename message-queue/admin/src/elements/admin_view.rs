use crate::elements::connection_interface::{ConnectionInterface, ConnectionInterfaceMessage};
use crate::elements::inspect_view::{InspectView, InspectViewMessage};
use crate::elements::{QueueView, UIMessage};
use crate::server_connector::ServerConnector;
use backend::protocol::new::queue_id::QueueId;
use backend::protocol::request::{DeleteQueue, GetProperties};
use backend::protocol::{BufferProperties, Status};
use iced::widget::{column, vertical_space};
use iced::{Element, Task};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Debug)]
pub enum AdminViewMessage {
    InspectBuffer(QueueId),
    InspectInfo(QueueId, BufferProperties),
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

pub struct AdminView {
    connector: Arc<Mutex<ServerConnector>>,

    // Sub-widgets
    buffer_view: QueueView,
    inspect_view: InspectView,
    connection_interface: ConnectionInterface,
}

impl Default for AdminView {
    fn default() -> Self {
        Self {
            connector: Arc::new(Mutex::new(ServerConnector::new())),
            buffer_view: QueueView::default(),
            inspect_view: InspectView::new(),
            connection_interface: ConnectionInterface::new(),
        }
    }
}

impl AdminView {
    pub fn view(&self) -> Element<AdminViewMessage> {
        match &self.inspect_view.buffer_info {
            Some(_) => self.inspect_view.view(),
            None => {
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
            AdminViewMessage::Inspector(m) => Task::batch([
                if let InspectViewMessage::Delete = m {
                    if let Some((addr, _)) = &self.inspect_view.buffer_info {
                        let connector = self.connector.clone();
                        let addr2 = addr.clone();
                        Task::perform(
                            async move { Self::delete_buffer(connector, addr2).await },
                            move |result| {
                                AdminViewMessage::ConnectionUpdated(
                                    ConnectionInterfaceMessage::Connected(result.is_some()),
                                )
                            },
                        )
                        .chain(
                            self.buffer_view
                                .update(UIMessage::Refresh, self.connector.clone()),
                        )
                    } else {
                        Task::none()
                    }
                } else {
                    Task::none()
                },
                {
                    self.inspect_view.update(m);
                    Task::none()
                },
            ]),
            AdminViewMessage::InspectBuffer(address) => {
                let connection = self.connector.clone();

                Task::perform(
                    async move {
                        if let Ok(client) = connection.lock().await.client().await {
                            let address_2 = address.clone();
                            Some((
                                address_2,
                                client
                                    .transfer_admin_request(GetProperties { buffer: address })
                                    .await,
                            ))
                        } else {
                            None
                        }
                    },
                    move |maybe_data| match maybe_data {
                        Some((address, properties)) => {
                            AdminViewMessage::InspectInfo(address, properties.unwrap())
                        }
                        None => AdminViewMessage::ConnectionUpdated(
                            ConnectionInterfaceMessage::Connected(false),
                        ),
                    },
                )
            }
            AdminViewMessage::InspectInfo(address, properties) => {
                self.inspect_view.buffer_info = Some((address, properties));
                Task::none()
            }
            AdminViewMessage::ConnectionUpdated(m) => {
                self.connection_interface.update(m, self.connector.clone())
            }
            _ => Task::none(),
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
