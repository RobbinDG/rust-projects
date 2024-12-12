use crate::elements::QueueView;

mod server_connector;
mod elements;

fn main() -> iced::Result {
    iced::run(
        "Message Queue Admin Panel",
        QueueView::update,
        QueueView::view,
    )
}
