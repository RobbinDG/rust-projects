mod server_connector;
use crate::elements::admin_view::AdminView;
mod elements;
mod fonts;
mod util;
pub mod make_request;

fn main() -> iced::Result {
    env_logger::init();

    iced::application(
        "Message Queue Admin Panel",
        AdminView::update,
        AdminView::view,
    )
    .font(iced_fonts::REQUIRED_FONT_BYTES)
    .font(iced_fonts::NERD_FONT_BYTES)
    .run()
}
