mod server_connector;
use crate::elements::admin_view::AdminView;
mod elements;

fn main() -> iced::Result {
    iced::run(
        "Message Queue Admin Panel",
        AdminView::update,
        AdminView::view,
    )
}
