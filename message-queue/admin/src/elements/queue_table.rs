use crate::elements::UIMessage;
use backend::protocol::BufferAddress;
use iced::widget::{button, column, horizontal_rule, row, scrollable, text, Column, Row};
use iced::{font, Alignment, Length};
use std::iter::zip;

pub struct QueueTable {
    names: [&'static str; 4],
    widths: [u16; 4],
    content: Vec<(BufferAddress, [String; 3])>,
    header_font: font::Font,
    height: Length,
}

impl QueueTable {
    pub fn new(names: [&'static str; 4], widths: [u16; 4]) -> Self {
        Self {
            names,
            widths,
            content: vec![],
            header_font: font::Font {
                weight: font::Weight::Bold,
                ..font::Font::DEFAULT
            },
            height: Length::Fill,
        }
    }

    pub fn clear(&mut self) {
        self.content.clear();
    }

    pub fn push(&mut self, row: (BufferAddress, usize, usize, usize)) {
        self.content.push((
            row.0,
            [row.1.to_string(), row.2.to_string(), row.3.to_string()],
        ));
    }

    pub fn view(&self) -> Column<UIMessage> {
        let header: Row<UIMessage> =
            row(zip(self.names, self.widths.clone()).map(|(name, width)| {
                text!("{}", name)
                    .width(width)
                    .align_x(Alignment::Center)
                    .font(self.header_font)
                    .into()
            }));
        let divider = horizontal_rule(2);
        let mut rows_column: Column<UIMessage> = column![]
            .spacing(2);
        if self.content.len() <= 0 {
            rows_column = rows_column.push(
                text("Nothing to see...")
                    .width(Length::Fill)
                    .align_x(Alignment::Center),
            );
        } else {
            for row_content in &self.content {
                let r = self.make_content_row(row_content);
                rows_column = rows_column.push(r);
            }
        }
        column![header, divider, scrollable(rows_column).width(Length::Fill)].height(self.height)
    }

    fn make_content_row(&self, row_content: &(BufferAddress, [String; 3])) -> Row<UIMessage> {
        let rows: [String; 4] = std::array::from_fn(|i| {
            if i == 0 {
                row_content.0.to_string()
            } else {
                row_content.1[i - 1].clone()
            }
        });
        let mut r: Row<UIMessage> =
            row(zip(self.widths, rows).map(|(w, c)| text(c).width(w).into()));
        r = r.push(button("Delete").on_press(UIMessage::DeleteQueue(row_content.0.clone())));
        r
    }

    pub fn height<T: Into<Length>>(mut self, height: T) -> Self {
        self.height = height.into();
        self
    }
}
