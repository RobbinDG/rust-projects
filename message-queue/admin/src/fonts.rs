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

pub const ELEMENT_SPACING_HORIZONTAL: u16 = 10;
pub const ELEMENT_SPACING_VERTICAL: u16 = 4;
