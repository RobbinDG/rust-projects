use iced::widget::{Button, Column, Row, Text};
use iced::{Alignment, Element, Length};

#[derive(Debug, Clone, Copy)]
pub enum Message<B> {
    Toggle,
    Body(B),
}

pub struct Collapsible {
    title: String,
    is_expanded: bool,
}

impl Collapsible {
    pub fn new(title: String, is_expanded: bool) -> Self {
        Self {
            title,
            is_expanded,
        }
    }

    pub fn toggle(&mut self) {
        self.is_expanded = !self.is_expanded;
    }

    pub fn view<'a, MessageBody: 'static>(
        &'a self,
        view_body: impl FnOnce() -> Element<'a, MessageBody>,
    ) -> Element<'a, Message<MessageBody>>
    where
        MessageBody: Clone,
    {
        let header = Button::new(
            Row::new()
                .spacing(20)
                .align_y(Alignment::Center)
                .push(Text::new(&self.title).width(Length::Fill))
                .push(Text::new(if self.is_expanded { "v" } else { "<" })),
        )
        .on_press(Message::Toggle);

        if self.is_expanded {
            Column::with_children(vec![
                header.into(),
                view_body().map(|msg| Message::Body(msg)),
            ])
            .into()
        } else {
            header.into()
        }
    }
}

impl From<Collapsible> for Element<'_, Message<Collapsible>> {
    fn from(value: Collapsible) -> Self {
        todo!()
    }
}
