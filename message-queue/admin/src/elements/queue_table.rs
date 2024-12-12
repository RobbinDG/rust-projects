use iced::widget::{button, column, row, text, Column, Row};
use std::iter::zip;
use iced::Alignment;
use crate::elements::UIMessage;

pub struct QueueTable {
    names: [&'static str; 4],
    widths: [u16; 4],
    content: Vec<[String; 4]>,
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

    pub fn push(&mut self, row: (String, usize, usize, usize)) {
        self.content.push([
            row.0,
            row.1.to_string(),
            row.2.to_string(),
            row.3.to_string(),
        ]);
        println!("content {:?}", self.content);
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
                let mut r: Row<UIMessage> =
                    row(zip(self.widths, row_content).map(|(w, c)| text(c).width(w).into()));
                r = r.push(
                    button("Delete").on_press(UIMessage::DeleteQueue(row_content[0].clone())),
                );
                column = column.push(r);
            }
        }
        column
    }
}