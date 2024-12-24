use crate::server_connector::ServerConnector;
use iced::widget::{button, row, text_input};
use iced::Element;
use std::net::SocketAddr;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum ConnectionInterfaceMessage {
    DesiredAddressChanged(String),
    Reconnect,
}
pub struct ConnectionInterface {
    entered_address: String,
    entered_address_valid: bool,
}

impl ConnectionInterface {
    pub fn new() -> Self {
        Self {
            entered_address: "".into(),
            entered_address_valid: false,
        }
    }

    pub fn view<'a, Message>(&'a self) -> Element<'a, Message>
    where
        Message: From<ConnectionInterfaceMessage> + Clone + 'a,
    {
        let mut reconnect_btn = button("Reconnect");
        if self.entered_address_valid {
            reconnect_btn = reconnect_btn.on_press(ConnectionInterfaceMessage::Reconnect);
        }
        let element: Element<ConnectionInterfaceMessage> = row![
            text_input("Address", self.entered_address.as_str())
                .on_input(ConnectionInterfaceMessage::DesiredAddressChanged),
            reconnect_btn,
        ]
        .into();
        element.map(Message::from)
    }

    pub fn update(&mut self, message: ConnectionInterfaceMessage, connector: &mut ServerConnector) {
        match message {
            ConnectionInterfaceMessage::DesiredAddressChanged(desired_address) => {
                self.entered_address_valid = SocketAddr::from_str(desired_address.as_str()).is_ok();
                self.entered_address = desired_address;
            }
            ConnectionInterfaceMessage::Reconnect => {
                if self.entered_address_valid {
                    connector.connect_to(self.entered_address.clone());
                }
            }
        }
    }
}
