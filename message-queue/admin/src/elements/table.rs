use iced::Element;
use iced_aw::{grid, grid_row};

pub struct Table<'a, M> {
    data: Vec<(Element<'a, M>, Element<'a, M>)>,
}

impl<'a, M> Table<'a, M> {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn push(mut self, left: impl Into<Element<'a, M>>, right: impl Into<Element<'a, M>>) -> Self {
        self.data.push((left.into(), right.into()));
        self
    }
}

impl<'a, M: 'static> From<Table<'a, M>> for Element<'a, M> {
    fn from(value: Table<'a, M>) -> Self {
        let mut grid = grid!().padding(2);
        for (left, right) in value.data {
            grid = grid.push(grid_row!(left, right));
        };
        grid.into()
    }
}
