mod server_connector;
use crate::elements::admin_view::AdminView;
mod elements;
mod fonts;
mod util;

fn main() -> iced::Result {
    iced::application(
        "Message Queue Admin Panel",
        AdminView::update,
        AdminView::view,
    )
    .font(iced_fonts::REQUIRED_FONT_BYTES)
    .font(iced_fonts::NERD_FONT_BYTES)
    .run()
}
