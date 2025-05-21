use gloo_net::http::Request;
use log::info;
use serde::Deserialize;
use yew::prelude::*;

const API_URL: &str = "http://127.0.0.1:8000";

#[derive(Deserialize)]
struct TransactionWithCategory {
    IBAN: String,
    currency: String,
    BIC: String,
    MTCN: i64,
    date: String,
    interest_date: String,
    value: f64,
    balance_after: f64,
    IBAN_other: Option<String>,
    name_other: String,
    BIC_other: Option<String>,
    code: Option<String>,
    reference: Option<String>,
    description: Option<String>,
    value_orig: Option<f64>,
    currency_orig: Option<String>,
    exchange_rate: Option<f64>,
    category: Option<String>,
}

#[function_component(TransactionsTable)]
pub fn app() -> Html {
    let users = use_state(|| Vec::<TransactionWithCategory>::new());
    let users_clone = users.clone();
    let hovered_row = use_state(|| None);

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
            users_clone.set(fetched);
        });
        || ()
    });

    let tooltip = {
        if let Some((id, x, y)) = *hovered_row {
            if let Some(row) = users.iter().find(|r| r.MTCN == id) {
                let iban_other = match &row.IBAN_other {
                    Some(d) => {
                        if d.is_empty() {
                            { "Unknown" }
                        } else {
                            d
                        }
                    },
                    None => { "Unknown" }
                };
                let desc = match &row.description {
                    Some(d) => {
                        if d.is_empty() {
                            { "No Description" }
                        } else {
                            d
                        }
                    },
                    None => "No Description",
                };
                html! {
                    <div
                        class="fixed z-50 bg-white text-sm text-gray-800 border rounded shadow-lg p-2"
                        style={format!("left: {}px; top: {}px;", x + 10, y + 10)}
                    >
                        <div class="font-semibold">{"Details"}</div>
                        <div>{"From: "}{ iban_other }</div>
                        <div>{"Description: "}{ desc }</div>
                        <div>{"Balance after: €"}{ &row.balance_after }</div>
                    </div>
                }
            } else {
                html! {}
            }
        } else {
            html! {}
        }
    };

    let rows = users.iter().map(|transaction| {
        let hovered_row_clone = hovered_row.clone();
        let id = transaction.MTCN;
        let on_mouse_enter = Callback::from(move |e: MouseEvent| {
            hovered_row_clone.set(Some((id, e.client_x(), e.client_y())));
        });

        let hovered_row = hovered_row.clone();
        let on_mouse_leave = Callback::from(move |_| {
            hovered_row.set(None);
        });

        html! {
            <tr
                class="border-b hover:bg-gray-100 relative"
                onmouseenter={on_mouse_enter}
                onmouseleave={on_mouse_leave}
            >
                <td class={classes!(
                    "px-4",
                    "py-2",
                    if transaction.value < 0.0 { "text-red-700" } else { "text-green-700" },
                )} style="text-align: right">{ '€' }{ format!("{:.02}", transaction.value.abs()) }</td>
                <td class="px-4 py-2">{ &transaction.name_other.clone() }</td>
                <td class="px-4 py-2">{ &transaction.category.clone() }</td>
            </tr>
        }
    });

    html! {
        <>
        { tooltip }
        <div class="p-4 max-h-screen overflow-auto">
            <div class="min-w-full overflow-x-auto border rounded shadow-md">
                <table class="min-w-full table-auto bg-white">
                    <thead class="bg-gray-200 sticky top-0 z-10">
                        <tr>
                            <th class="px-4 py-2 text-left">{ "Value" }</th>
                            <th class="px-4 py-2 text-left">{ "Name" }</th>
                            <th class="px-4 py-2 text-left">{ "Category" }</th>
                        </tr>
                    </thead>
                    <tbody>
                        { for rows }
                    </tbody>
                </table>
            </div>
        </div>
        </>
    }
}
