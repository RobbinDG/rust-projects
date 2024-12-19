use std::net::SocketAddr;
use std::str::FromStr;
use crate::server_connector::ServerConnector;
use iced::widget::{button, row, text_input, Row};

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

    pub fn view<'a, Message>(&'a self) -> Row<'a, Message>
    where
        Message: From<ConnectionInterfaceMessage> + Clone + 'a,
    {
        let mut reconnect_btn = button("Reconnect");
        if self.entered_address_valid {
            reconnect_btn = reconnect_btn.on_press(Message::from(ConnectionInterfaceMessage::Reconnect));
        }
        row![
            text_input("Address", self.entered_address.as_str()).on_input(move |input: String| {
                Message::from(ConnectionInterfaceMessage::DesiredAddressChanged(input))
            }),
            reconnect_btn,
        ]
    }

    pub fn update(&mut self, message: ConnectionInterfaceMessage, connector: &ServerConnector) {
        match message {
            ConnectionInterfaceMessage::DesiredAddressChanged(desired_address) => {
                self.entered_address_valid = SocketAddr::from_str(desired_address.as_str()).is_ok();
                self.entered_address = desired_address;
            }
            ConnectionInterfaceMessage::Reconnect => {
                if self.entered_address_valid {
                    if connector.connected() {
                        // connector.
                        todo!()
                    }
                }
            }
        }
    }
}
