use crate::aggregates_table::AggregatesTable;
use crate::transactions_page::TransactionsPage;
use yew::{html, Html};
use yew_router::prelude::*;

#[derive(Routable, PartialEq, Clone)]
pub enum AppRoute {
    #[at("/")]
    Overview,
    #[at("/transactions")]
    Transactions,
}

pub fn switch(route: AppRoute) -> Html {
    match route {
        AppRoute::Overview => html! { <AggregatesTable /> },
        AppRoute::Transactions => html! { <TransactionsPage /> },
    }
}
