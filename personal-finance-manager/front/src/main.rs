mod app;
mod aggregates_table;

use gloo::utils::document;
use app::TransactionsTable;
use aggregates_table::AggregatesTable;

fn main() {
    yew::Renderer::<TransactionsTable>::with_root(document().get_element_by_id("transactions").unwrap()).render();
    yew::Renderer::<AggregatesTable>::with_root(document().get_element_by_id("aggregates").unwrap()).render();
}
