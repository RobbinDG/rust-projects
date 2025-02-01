use std::collections::HashSet;
use crate::elements::collapsible;
use crate::elements::collapsible::Collapsible;
use backend::protocol::queue_id::TopicLiteral;
use iced::widget::{button, container, hover, mouse_area, row, text, text_input, Container, Row};
use iced::{color, Background, Border, Element, Length, Padding};

#[derive(Debug, Clone)]
pub enum Message {
    CreateSubtopic(String, Option<String>),
    NewSubtopicNameChanged(String),
    ToggleBreakdown(Option<usize>),
    SubSubSelected(usize, usize),
}

pub struct TopicBreakdown {
    breakdown_view: Collapsible,
    sub_breakdown_views: Vec<(String, Collapsible, Vec<String>)>,
    new_subtopic_name: String,
    subsub_selection: Option<(usize, usize)>,
}

impl TopicBreakdown {
    pub fn subsubs(&self, sub: &TopicLiteral) -> Vec<TopicLiteral> {
        let mut topics = HashSet::new();
        match &sub {
            TopicLiteral::Name(s) => {
                for (a, _, b) in &self.sub_breakdown_views {
                    if a == s {
                        topics.extend(b.iter().map(|s| TopicLiteral::Name(s.clone())));
                    }
                }
            }
            TopicLiteral::Wildcard => {
                for (_, _, b) in &self.sub_breakdown_views {
                    topics.extend(b.iter().map(|s| TopicLiteral::Name(s.clone())));
                }
            }
        };
        topics.into_iter().collect()
    }
}

impl TopicBreakdown {
    pub fn new(title: String) -> Self {
        Self {
            breakdown_view: Collapsible::new(title, false),
            sub_breakdown_views: Vec::new(),
            new_subtopic_name: String::new(),
            subsub_selection: None,
        }
    }

    pub fn view(&self) -> Element<Message> {
        self.breakdown_view
            .view(|| self.build_subtopic_view())
            .map(|msg| match msg {
                collapsible::Message::Toggle => Message::ToggleBreakdown(None),
                collapsible::Message::Body(msg) => msg,
            })
            .into()
    }

    pub fn update(&mut self, msg: Message) {
        match msg {
            Message::CreateSubtopic(_, _) => {}
            Message::NewSubtopicNameChanged(s) => self.new_subtopic_name = s,
            Message::ToggleBreakdown(which) => match which {
                None => self.breakdown_view.toggle(),
                Some(i) => {
                    if let Some((_, c, _)) = self.sub_breakdown_views.get_mut(i) {
                        c.toggle()
                    }
                }
            },
            Message::SubSubSelected(i, j) => self.subsub_selection = Some((i, j)),
        }
    }

    pub fn selected_topic(&self) -> Option<(TopicLiteral, TopicLiteral)> {
        let (i, j) = self.subsub_selection?;
        let (name, _, subs) = self.sub_breakdown_views.get(i)?;
        Some((
            TopicLiteral::Name(name.clone()),
            TopicLiteral::Name(subs.get(j)?.clone()),
        ))
    }

    pub fn set_data(&mut self, data: Vec<(String, Vec<String>)>) {
        self.sub_breakdown_views.clear();
        for (s, ss) in data {
            self.sub_breakdown_views
                .push((s.clone(), Collapsible::new(s, false), ss))
        }
    }

    fn build_subtopic_view(&self) -> Element<Message> {
        let mut col = iced::widget::column![].padding(Padding::ZERO.left(10));
        let mut e = self.sub_breakdown_views.iter().enumerate();
        while let Some((i, (_, c, s))) = e.next() {
            col = col.push(c.view(|| self.build_subsubtopic_view(s, i)).map(
                move |msg| match msg {
                    collapsible::Message::Toggle => Message::ToggleBreakdown(Some(i)),
                    collapsible::Message::Body(m) => m,
                },
            ));
        }
        col = col.push(self.build_create_prompt(None));
        col.into()
    }

    fn build_create_prompt(&self, which: Option<usize>) -> Row<Message> {
        let message = match which {
            None => Some(Message::CreateSubtopic(
                self.new_subtopic_name.clone(),
                None,
            )),
            Some(i) => match self.sub_breakdown_views.get(i) {
                None => None,
                Some((s, _, _)) => Some(Message::CreateSubtopic(
                    s.clone(),
                    Some(self.new_subtopic_name.clone()),
                )),
            },
        };
        let mut btn = button("Create");
        if let Some(msg) = message {
            btn = btn.on_press(msg);
        }
        row![
            text_input("New subtopic", &self.new_subtopic_name)
                .on_input(Message::NewSubtopicNameChanged),
            btn,
        ]
    }

    fn build_subsubtopic_view<'a>(&'a self, s: &'a Vec<String>, i: usize) -> Element<'a, Message> {
        let mut col = iced::widget::column![].padding(Padding::ZERO.left(10));
        for (i_sub, ss) in s.iter().enumerate() {
            let text_elem: Element<Message> = if Some((i, i_sub)) == self.subsub_selection {
                Self::format_container(container(text(ss))).into()
            } else {
                text(ss).into()
            };
            col = col.push(
                mouse_area(hover(
                    text_elem,
                    Self::format_container(container(""))
                        .width(Length::Fill)
                        .height(Length::Fill),
                ))
                .on_press(Message::SubSubSelected(i, i_sub)),
            );
        }
        col = col.push(self.build_create_prompt(Some(i)));
        col.into()
    }

    fn format_container<M>(container: Container<M>) -> Container<M> {
        container.style(|t| {
            container::rounded_box(t)
                .border(Border {
                    color: color![0.1, 0.1, 1.0, 0.8],
                    width: 2.0,
                    radius: Default::default(),
                })
                .background(Background::Color(color![0.1, 0.1, 1.0, 0.5]))
        })
    }
}
