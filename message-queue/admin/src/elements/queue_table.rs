use crate::elements::queue_view::UIMessage;
use backend::protocol::queue_id::TopLevelQueueId;
use iced::widget::{
    column, container, horizontal_rule, hover, mouse_area, row, scrollable, text, Column, Row,
};
use iced::{color, font, Alignment, Background, Border, Element, Length};
use std::iter::zip;

pub struct QueueTable {
    names: [&'static str; 3],
    widths: [u16; 3],
    content: Vec<(TopLevelQueueId, [String; 2])>,
    header_font: font::Font,
    height: Length,
}

impl QueueTable {
    pub fn new(names: [&'static str; 3], widths: [u16; 3]) -> Self {
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

    pub fn push(&mut self, row: (TopLevelQueueId, usize, usize)) {
        self.content.push((
            row.0.into(),
            [row.1.to_string(), row.2.to_string()],
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
        let mut rows_column: Column<UIMessage> = column![].spacing(2);
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

    fn make_content_row(&self, row_content: &(TopLevelQueueId, [String; 2])) -> Element<UIMessage> {
        let rows: [String; 3] = std::array::from_fn(|i| {
            if i == 0 {
                row_content.0.to_string()
            } else {
                row_content.1[i - 1].clone()
            }
        });
        let r: Row<UIMessage> = row(zip(self.widths, rows).map(|(w, c)| text(c).width(w).into()));
        mouse_area(hover(
            r,
            container("")
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|t| {
                    container::rounded_box(t)
                        .border(Border {
                            color: color![0.1, 0.1, 1.0, 0.8],
                            width: 2.0,
                            radius: Default::default(),
                        })
                        .background(Background::Color(color![0.1, 0.1, 1.0, 0.5]))
                }),
        ))
        .on_press(UIMessage::InspectBuffer(row_content.0.clone()))
        .into()
    }
}
