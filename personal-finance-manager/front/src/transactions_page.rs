use gloo_net::http::Request;
use serde::Deserialize;
use yew::prelude::*;
use crate::transactions_table::TransactionsTable;

const API_URL: &str = "http://127.0.0.1:8000";

#[derive(Deserialize, PartialEq, Clone)]
pub struct TransactionWithCategory {
    pub IBAN: String,
    pub currency: String,
    pub BIC: String,
    pub MTCN: i64,
    pub date: String,
    pub interest_date: String,
    pub value: f64,
    pub balance_after: f64,
    pub IBAN_other: Option<String>,
    pub name_other: String,
    pub BIC_other: Option<String>,
    pub code: Option<String>,
    pub reference: Option<String>,
    pub description: Option<String>,
    pub value_orig: Option<f64>,
    pub currency_orig: Option<String>,
    pub exchange_rate: Option<f64>,
    pub category: Option<String>,
}

#[function_component(TransactionsPage)]
pub fn overview_page() -> Html {
    let transactions = use_state(Vec::<TransactionWithCategory>::new);

    {
        let transactions = transactions.clone();

        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let fetched: Vec<TransactionWithCategory> =
                    Request::get((API_URL.to_owned() + "/transactions").as_str())
                        .send()
                        .await
                        .unwrap()
                        .json()
                        .await
                        .unwrap();
                transactions.set(fetched);
            });
            || ()
        });
    }
    html! { <TransactionsTable transactions={(*transactions).clone()} /> }
}
