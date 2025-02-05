use backend::protocol::queue_id::{QueueFilter, QueueId, TopicLiteral};
use iced::Element;
use crate::elements::topic_breakdown;

#[derive(Clone, Debug)]
pub enum Message {
    SubtopicCreateSelectionChanged0(TopicLiteral),
    SubtopicCreateSelectionChanged1(TopicLiteral),
    BreakdownMessage(topic_breakdown::Message),
    BreakdownLoaded(Option<Vec<(String, Vec<String>)>>),
    CreateSubtopic(String, Option<String>),
}

pub trait QueueSelector {
    fn view(&self) -> impl Into<Element<Message>>;

    fn update(&mut self, message: Message);

    fn selected(&self) -> Option<QueueId>;

    fn selected_filter(&self) -> QueueFilter;
}
