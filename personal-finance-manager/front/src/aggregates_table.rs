use crate::info_panel::InfoPanel;
use crate::transactions_page::TransactionWithCategory;
use gloo_net::http::Request;
use serde::Deserialize;
use yew::prelude::*;
use crate::editable_table::EditableTable;

pub const API_URL: &str = "http://127.0.0.1:8000";

#[derive(Deserialize)]
struct MonthAggregate {
    year_month: String,
    sum: f64,
}

#[function_component(AggregatesTable)]
pub fn app() -> Html {
    let users = use_state(|| Vec::<MonthAggregate>::new());
    let users_clone = users.clone();

    use_effect_with((), move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            let fetched: Vec<MonthAggregate> =
                Request::get((API_URL.to_owned() + "/aggregates").as_str())
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

    let selected_id = use_state(|| None::<String>);
    let selected_data = use_state(|| None::<RowData>);
    let selected_transactions = use_state(|| Vec::new());

    {
        let selected_id = selected_id.clone();
        let selected_data = selected_data.clone();
        let selected_transactions = selected_transactions.clone();

        use_effect_with((*selected_id).clone(), move |id| {
            let id = id.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if let Some(id) = id {
                    let items: RowData =
                        Request::get((API_URL.to_owned() + "/breakdowns/" + id.as_str()).as_str())
                            .send()
                            .await
                            .unwrap()
                            .json()
                            .await
                            .unwrap();
                    selected_data.set(Some(items));

                    let items: Vec<TransactionWithCategory> =
                        Request::get((API_URL.to_owned() + "/transactions/" + id.as_str()).as_str())
                            .send()
                            .await
                            .unwrap()
                            .json()
                            .await
                            .unwrap();
                    selected_transactions.set(items);
                }
            });
            || ()
        });
    }

    let agg_rows = users.iter().map(|agg| {
        let year_month = agg.year_month.clone();
        let on_row_click = {
            let selected_id = selected_id.clone();
            Callback::from(move |_| {
                selected_id.set(Some(year_month.clone()));
            })
        };
        html! {
            <tr
                class="relative"
                onclick={on_row_click}
            >
                <td>{ &agg.year_month }</td>
                <td class={classes!(
                    if agg.sum < 0.0 { "text-red-700" } else { "text-green-700" },
                )} style="text-align: right">{ '€' }{ format!("{:.02}", agg.sum.abs()) }</td>
            </tr>
        }
    });

    let cat_rows = match &*selected_data {
        None => vec![],
        Some(d) => d.items.clone(),
    };

    html! {
        <div class="flex flex-row gap-5">
            <div class="basis-xs shrink-0 flex flex-col space-y-5 w-full">
                <div class="container max-h-1/2 h-1/2">
                    <div class="table-container">
                        <table class="w-full">
                            <thead>
                                <tr>
                                    <th class="text-left">{ "Month" }</th>
                                    <th class="text-right">{ "Value" }</th>
                                </tr>
                            </thead>
                            <tbody>
                                { for agg_rows }
                            </tbody>
                        </table>
                    </div>
                </div>
                <div class="max-h-1/2 h-1/2 container overflow-auto">
                    <EditableTable sub_items={cat_rows} />
                </div>
            </div>
            <div class="basis-lg shrink-0">
                <InfoPanel selected_data={(*selected_data).clone()} transactions={(*selected_transactions).clone()} />
            </div>
        </div>
    }
}

// Mock main row data structure
#[derive(Clone, PartialEq, Deserialize)]
pub struct RowData {
    pub year_month: String,
    pub items: Vec<SubItem>,
}

#[derive(Clone, PartialEq, Deserialize)]
pub struct SubItem {
    pub category: String,
    pub breakdown_value: f64,
}