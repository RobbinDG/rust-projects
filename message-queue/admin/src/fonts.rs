use iced::font::Weight;
use iced::Font;

pub fn font_heading() -> Font {
    Font {
        family: Default::default(),
        weight: Weight::Bold,
        stretch: Default::default(),
        style: Default::default(),
    }
}
pub const SIZE_HEADING: u16 = 20;

pub const ELEMENT_SPACING: u16 = 10;
