use crate::elements::inspect_view::{InspectView, InspectViewMessage};
use crate::elements::{QueueView, UIMessage};
use iced::Element;

#[derive(Clone, Debug)]
pub enum AdminViewMessage {
    BufferView(UIMessage),
    Inspector(InspectViewMessage),
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
    selected_buffer: Option<()>,

    buffer_view: QueueView,
    inspect_view: InspectView,
}

impl Default for AdminView {
    fn default() -> Self {
        Self {
            selected_buffer: None,
            buffer_view: QueueView::default(),
            inspect_view: InspectView::new(),
        }
    }
}

impl AdminView {
    pub fn view(&self) -> Element<AdminViewMessage> {
        if self.selected_buffer.is_some() {
            self.inspect_view.view()
        } else {
            self.buffer_view.view()
        }
    }

    pub fn update(&mut self, message: AdminViewMessage) {
        match message {
            AdminViewMessage::BufferView(m) => self.buffer_view.update(m),
            AdminViewMessage::Inspector(m) => self.inspect_view.update(m),
        }
    }
}
