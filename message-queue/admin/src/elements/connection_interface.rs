use crate::server_connector::ServerConnector;
use iced::widget::{button, container, row, text, text_input};
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
    pub fn new() -> Self {
        Self {
            connected: false,
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
        let mut row = row![
            text_input("Address", self.entered_address.as_str())
                .on_input(ConnectionInterfaceMessage::DesiredAddressChanged),
            reconnect_btn,
        ];
        if !self.connected {
            row = row.push(
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

        let y = <Element<ConnectionInterfaceMessage>>::from(row);
        // let x = row as ;
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
                self.entered_address_valid = SocketAddr::from_str(desired_address.as_str()).is_ok();
                self.entered_address = desired_address;
                Task::none()
            }
            ConnectionInterfaceMessage::Reconnect => {
                if self.entered_address_valid {
                    let queue_id = self.entered_address.clone();
                    Task::perform(
                        async move {
                            connector.lock().await.connect_to(queue_id).await;
                        },
                        |_| ConnectionInterfaceMessage::Connected(true).into(),
                    )
                } else {
                    Task::none()
                }
            }
            ConnectionInterfaceMessage::Connected(_) => {
                self.connected = true;
                Task::none()
            }
        }
    }
}
