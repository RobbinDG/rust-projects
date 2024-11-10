pub trait Interface {
    fn print_query(&self) {
        println!("What shall we do?");
        self.print_options();
    }
    fn print_options(&self);

    fn on_selection(self: Box<Self>, choice: u32) -> Box<dyn Interface>;
}