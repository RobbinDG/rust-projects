use backend::protocol::BufferAddress;
use iced::widget::{button, column, row, text};
use iced::{Alignment, Element, Length};

#[derive(Clone, Debug)]
pub enum InspectViewMessage {
    Close,
    Delete,
}

pub struct InspectView {
    pub buffer_info: Option<BufferAddress>,
}

impl InspectView {
    pub fn new() -> Self {
        Self { buffer_info: None }
    }

    pub fn view<'a, Message>(&'a self) -> Element<'a, Message>
    where
        Message: From<InspectViewMessage> + 'a,
    {
        match &self.buffer_info {
            Some(buffer_info) => {
                let element: Element<InspectViewMessage> = column![
                    row![
                        button("<").on_press(InspectViewMessage::Close),
                        text(format![
                            "{:?} {}",
                            buffer_info.buffer_type(),
                            buffer_info.to_string()
                        ])
                        .width(Length::Fill)
                        .align_x(Alignment::Center),
                    ],
                    button("Delete").on_press(InspectViewMessage::Delete),
                ]
                .into();
                element.map(Message::from)
            }
            None => text("No buffer selected.").into(),
        }
    }

    pub fn update(&mut self, message: InspectViewMessage) {
        match message {
            InspectViewMessage::Delete => self.buffer_info = None,
            InspectViewMessage::Close => self.buffer_info = None,
        }
    }
}
