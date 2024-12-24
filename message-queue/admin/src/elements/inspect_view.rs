use iced::widget::button;
use iced::Element;

#[derive(Clone, Debug)]
pub enum InspectViewMessage {
    Delete,
}

pub struct InspectView {}

impl InspectView {
    pub fn new() -> Self {
        Self {}
    }

    pub fn view<'a, Message>(&'a self) -> Element<'a, Message>
    where
        Message: From<InspectViewMessage> + 'a,
    {
        let element: Element<InspectViewMessage> =
            button("Delete").on_press(InspectViewMessage::Delete).into();
        element.map(Message::from)
    }

    pub fn update(&mut self, message: InspectViewMessage) {}
}
