use backend::protocol::{BufferAddress, BufferProperties};
use iced::widget::{button, column, row, text};
use iced::{Alignment, Element, Length};

#[derive(Clone, Debug)]
pub enum InspectViewMessage {
    Close,
    Delete,
}

pub struct InspectView {
    pub buffer_info: Option<(BufferAddress, BufferProperties)>,
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
            Some((address, properties)) => {
                let mut delete_btn = button("Delete");
                if !properties.system_buffer {
                    delete_btn = delete_btn.on_press(InspectViewMessage::Delete);
                }
                let element: Element<InspectViewMessage> = column![
                    row![
                        button("<").on_press(InspectViewMessage::Close),
                        text(format![
                            "{:?} {}",
                            address.buffer_type(),
                            address.to_string()
                        ])
                        .width(Length::Fill)
                        .align_x(Alignment::Center),
                    ],
                    delete_btn,
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
