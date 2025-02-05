use crate::elements::queue_selector::{Message, QueueSelector};
use crate::elements::topic_breakdown;
use crate::elements::topic_breakdown::TopicBreakdown;
use crate::fonts::{ELEMENT_SPACING_HORIZONTAL, ELEMENT_SPACING_VERTICAL};
use backend::protocol::queue_id::{QueueFilter, QueueId, TopicLiteral};
use iced::widget::{combo_box, text};
use iced::widget::{Column, Row};
use iced::Element;

pub struct TopicSelector {
    name: String,
    breakdown_view: TopicBreakdown,
    new_filter_state: (
        combo_box::State<TopicLiteral>,
        combo_box::State<TopicLiteral>,
    ),
    new_filter_selection: (Option<TopicLiteral>, Option<TopicLiteral>),
}

impl TopicSelector {
    pub fn new(name: String) -> Self {
        Self {
            name,
            breakdown_view: TopicBreakdown::new("Breakdown".into()),
            new_filter_state: (combo_box::State::new(vec![]), combo_box::State::new(vec![])),
            new_filter_selection: (None, None),
        }
    }
}

impl QueueSelector for TopicSelector {
    fn view(&self) -> impl Into<Element<Message>> {
        Column::new()
            .spacing(ELEMENT_SPACING_VERTICAL)
            .push(self.breakdown_view.view().map(|msg| match msg {
                topic_breakdown::Message::CreateSubtopic(s, ss) => Message::CreateSubtopic(s, ss),
                m => Message::BreakdownMessage(m),
            }))
            .push(
                Row::new()
                    .spacing(ELEMENT_SPACING_HORIZONTAL)
                    .push(text("Topic Selection"))
                    .push(combo_box(
                        &self.new_filter_state.0,
                        "topic",
                        self.new_filter_selection.0.as_ref(),
                        Message::SubtopicCreateSelectionChanged0,
                    ))
                    .push(combo_box(
                        &self.new_filter_state.1,
                        "topic",
                        self.new_filter_selection.1.as_ref(),
                        Message::SubtopicCreateSelectionChanged1,
                    )),
            )
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::SubtopicCreateSelectionChanged0(choice) => {
                let topics = self.breakdown_view.subsubs(&choice);
                let mut items = vec![TopicLiteral::Wildcard];
                items.extend(topics);
                self.new_filter_state.1 = combo_box::State::new(items);
                self.new_filter_selection = (Some(choice), None);
            }
            Message::SubtopicCreateSelectionChanged1(choice) => {
                self.new_filter_selection.1 = Some(choice);
            }
            Message::BreakdownMessage(msg) => self.breakdown_view.update(msg),
            Message::BreakdownLoaded(breakdown) => {
                if let Some(data) = breakdown {
                    let topics: Vec<_> = data
                        .iter()
                        .map(|e| TopicLiteral::Name(e.0.clone()))
                        .collect();
                    let mut items = vec![TopicLiteral::Wildcard];
                    items.extend(topics);
                    self.new_filter_state = (
                        combo_box::State::new(items),
                        combo_box::State::new(vec![TopicLiteral::Wildcard]),
                    );
                    self.breakdown_view.set_data(data);
                }
            }
            _ => {}
        }
    }

    fn selected(&self) -> Option<QueueId> {
        match &self.new_filter_selection {
            (Some(TopicLiteral::Name(f1)), Some(TopicLiteral::Name(f2))) => {
                Some(QueueId::Topic(self.name.clone(), f1.clone(), f2.clone()))
            }
            _ => None,
        }
    }

    fn selected_filter(&self) -> QueueFilter {
        let f1 = match &self.new_filter_selection.0 {
            None => TopicLiteral::Wildcard,
            Some(lit) => lit.clone(),
        };
        let f2 = match &self.new_filter_selection.1 {
            None => TopicLiteral::Wildcard,
            Some(lit) => lit.clone(),
        };
        QueueFilter::Topic(self.name.clone(), f1, f2)
    }
}
