use crate::fonts::{ELEMENT_SPACING_HORIZONTAL, ELEMENT_SPACING_VERTICAL};
use crate::server_connector::ServerConnector;
use iced::widget::{button, container, text, text_input, Column, Row};
use iced::{Alignment, Element, Length, Task};
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub enum ConnectionInterfaceMessage {
    DesiredAddressChanged(String),
    Reconnect,
    Connected(bool),
}
pub struct ConnectionInterface {
    entered_address: String,
    entered_address_valid: bool,
    connected: bool,
}

impl ConnectionInterface {
    pub fn new(initial_address: String) -> Self {
        let valid = Self::validate_address(&initial_address);
        Self {
            connected: false,
            entered_address: initial_address,
            entered_address_valid: valid,
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
        let row = Row::new()
            .spacing(ELEMENT_SPACING_HORIZONTAL)
            .push(
                text_input("Address", self.entered_address.as_str())
                    .on_input(ConnectionInterfaceMessage::DesiredAddressChanged),
            )
            .push(reconnect_btn);
        let mut col = Column::new().spacing(ELEMENT_SPACING_VERTICAL).push(row);
        if !self.connected {
            col = col.push(
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

        let y = <Element<ConnectionInterfaceMessage>>::from(col);
        y.map(Message::from)
    }

    pub fn update<Message>(
        &mut self,
        message: ConnectionInterfaceMessage,
        connector: Arc<Mutex<ServerConnector>>,
    ) -> Task<Message>
    where
        Message: From<ConnectionInterfaceMessage> + Clone + 'static + Send,
    {
        match message {
            ConnectionInterfaceMessage::DesiredAddressChanged(desired_address) => {
                self.entered_address_valid = Self::validate_address(&desired_address);
                self.entered_address = desired_address;
                self.set_connected(false);
            }
            ConnectionInterfaceMessage::Reconnect => {
                if self.entered_address_valid {
                    let queue_id = self.entered_address.clone();
                    return Task::perform(
                        async move {
                            connector.lock().await.connect_to(queue_id).await
                        },
                        |connected| ConnectionInterfaceMessage::Connected(connected).into(),
                    );
                }
            }
            ConnectionInterfaceMessage::Connected(val) => {
                self.set_connected(val);
            }
        }
        Task::none()
    }

    fn validate_address(desired_address: &String) -> bool {
        SocketAddr::from_str(desired_address.as_str()).is_ok()
    }

    pub fn set_connected(&mut self, connected: bool) {
        self.connected = connected;
    }
}
