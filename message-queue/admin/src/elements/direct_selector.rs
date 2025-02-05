use crate::elements::queue_selector::{Message, QueueSelector};
use backend::protocol::queue_id::{QueueFilter, QueueId};
use iced::widget::row;
use iced::Element;

pub struct DirectSelector {
    name: String,
}

impl DirectSelector {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl QueueSelector for DirectSelector {
    fn view(&self) -> impl Into<Element<Message>> {
        row![]
    }

    fn update(&mut self, _: Message) {}

    fn selected(&self) -> Option<QueueId> {
        Some(QueueId::Queue(self.name.clone()))
    }

    fn selected_filter(&self) -> QueueFilter {
        QueueFilter::Queue(self.name.clone())
    }
}
