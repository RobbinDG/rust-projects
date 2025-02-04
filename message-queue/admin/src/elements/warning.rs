use iced::widget::{container, text, tooltip};
use iced::Element;
use iced_fonts::{nerd, Nerd};
use std::marker::PhantomData;

pub struct Warning<M> {
    message: String,
    t: PhantomData<M>,
}

impl<M> Warning<M> {
    pub fn new<S: Into<String>>(message: S) -> Self {
        Self {
            message: message.into(),
            t: PhantomData::default(),
        }
    }
}

impl<'a, M: 'a> From<Warning<M>> for Element<'a, M> {
    fn from(value: Warning<M>) -> Self {
        tooltip(
            text(nerd::icon_to_char(Nerd::Warning)).font(iced_fonts::NERD_FONT),
            container(text(value.message))
                .padding(10)
                .style(container::rounded_box),
            tooltip::Position::Bottom,
        )
        .into()
    }
}
