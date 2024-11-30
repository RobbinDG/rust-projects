use crate::interface::Interface;

pub struct SenderInterface {}

impl Interface for SenderInterface {
    fn print_options(&self) {
        todo!()
    }

    fn on_selection(self: Box<Self>, choice: u32) -> Box<dyn Interface> {
        todo!()
    }
}