use crate::elements::UIMessage;
use backend::protocol::BufferAddress;
use iced::widget::{button, column, row, text, Column, Row};
use iced::Alignment;
use std::iter::zip;

pub struct QueueTable {
    names: [&'static str; 4],
    widths: [u16; 4],
    content: Vec<(BufferAddress, [String; 3])>,
}

impl QueueTable {
    pub fn new(names: [&'static str; 4], widths: [u16; 4]) -> Self {
        Self {
            names,
            widths,
            content: vec![],
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
                    .into()
            }));
        let mut column: Column<UIMessage> = column![header];
        if self.content.len() <= 0 {
            column = column.push(
                text("Nothing to see...")
                    .width(500)
                    .align_x(Alignment::Center),
            );
        } else {
            for row_content in &self.content {
                let rows: [String; 4] = std::array::from_fn(|i| {
                    if i == 0 {
                        row_content.0.to_string()
                    } else {
                        row_content.1[i - 1].clone()
                    }
                });
                let mut r: Row<UIMessage> =
                    row(zip(self.widths, rows).map(|(w, c)| text(c).width(w).into()));
                r = r.push(
                    button("Delete").on_press(UIMessage::DeleteQueue(row_content.0.clone())),
                );
                column = column.push(r);
            }
        }
        column
    }
}
