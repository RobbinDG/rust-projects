use crate::elements::connection_interface::{ConnectionInterface, ConnectionInterfaceMessage};
use crate::elements::inspect_view::{InspectView, InspectViewMessage};
use crate::elements::{QueueView, UIMessage};
use crate::server_connector::ServerConnector;
use backend::protocol::request::{DeleteQueue, GetProperties};
use backend::protocol::BufferAddress;
use iced::widget::{column, container, text, vertical_space};
use iced::{Alignment, Element, Length};

#[derive(Clone, Debug)]
pub enum AdminViewMessage {
    InspectBuffer(BufferAddress),
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

pub struct AdminView {
    connector: ServerConnector,

    // Sub-widgets
    buffer_view: QueueView,
    inspect_view: InspectView,
    connection_interface: ConnectionInterface,
}

impl Default for AdminView {
    fn default() -> Self {
        Self {
            connector: ServerConnector::new(),
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
                if !self.connector.connected() {
                    cols = cols.push(
                        container(
                            text("Couldn't connect to server.")
                                .width(Length::Fill)
                                .align_x(Alignment::Center),
                        )
                        .width(Length::Fill)
                        .align_x(Alignment::Center)
                        .padding(10)
                        .style(container::rounded_box),
                    );
                }
                cols.into()
            }
        }
    }

    pub fn update(&mut self, message: AdminViewMessage) {
        match message {
            AdminViewMessage::BufferView(m) => self.buffer_view.update(m, &mut self.connector),
            AdminViewMessage::Inspector(m) => {
                println!("{m:?}");
                if let InspectViewMessage::Delete = m {
                    if let Some((addr, _)) = &self.inspect_view.buffer_info {
                        self.delete_buffer(addr.clone());
                        self.buffer_view.update(UIMessage::Refresh, &mut self.connector)
                    }
                }
                self.inspect_view.update(m)
            }
            AdminViewMessage::InspectBuffer(address) => {
                if let Ok(client) = self.connector.client() {
                    let properties = client.transfer_admin_request(GetProperties { buffer: address.clone() }).unwrap();
                    self.inspect_view.buffer_info = Some((address, properties));
                }
            }
            AdminViewMessage::ConnectionUpdated(m) => {
                self.connection_interface.update(m, &mut self.connector)
            }
        }
    }

    fn delete_buffer(&mut self, s: BufferAddress) {
        if let Ok(client) = self.connector.client() {
            client.transfer_admin_request(DeleteQueue { queue_name: s }).unwrap();
        }
    }
}
