mod app;
mod aggregates_table;
mod upload_transactions;

use gloo::utils::document;
use app::TransactionsTable;
use aggregates_table::AggregatesTable;
use crate::upload_transactions::UploadTransactions;

fn main() {
    yew::Renderer::<UploadTransactions>::with_root(document().get_element_by_id("upload-transaction-form").unwrap()).render();
    yew::Renderer::<TransactionsTable>::with_root(document().get_element_by_id("transactions").unwrap()).render();
    yew::Renderer::<AggregatesTable>::with_root(document().get_element_by_id("aggregates").unwrap()).render();
}
