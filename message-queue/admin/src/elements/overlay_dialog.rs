use iced::widget::{button, column, horizontal_rule, row, text};
use iced::{Alignment, Element, Length};
use std::fmt::Debug;

#[derive(Clone, Debug)]
pub enum Message<M>
where
    M: Clone + Debug,
{
    Close,
    Dialog(M),
}

pub struct OverlayDialog<B> {
    title: String,
    body: B,
}

impl<B> OverlayDialog<B> {
    pub fn new(title: String, body: B) -> Self {
        Self { title, body }
    }

    pub fn view<'a, M, F>(&'a self, view_func: F) -> Element<'a, Message<M>>
    where
        M: Clone + Debug + 'a,
        F: Fn(&B) -> Element<M>,
    {
        column![
            row![
                button("<").on_press(Message::Close),
                text(self.title.clone())
                    .width(Length::Fill)
                    .align_x(Alignment::Center),
            ],
            horizontal_rule(1),
            view_func(&self.body).map(Message::Dialog),
        ]
        .into()
    }

    pub fn update<'a, F, R>(&'a mut self, update_func: F) -> R
    where
        F: FnOnce(&'a mut B) -> R + 'a,
    {
        update_func(&mut self.body)
    }

    pub fn close(self) -> B {
        self.body
    }
}
