use std::process::id;
use gloo_net::http::Request;
use serde::Deserialize;
use yew::prelude::*;
use crate::info_panel::InfoPanel;

const API_URL: &str = "http://127.0.0.1:8000";

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

    let selected_id = use_state(|| Some("1".to_string())); // Always visible
    let selected_data = use_state(|| get_data_for_id("1".to_string())); // Dummy data fetch

    let on_row_click = {
        let selected_id = selected_id.clone();
        let selected_data = selected_data.clone();
        Callback::from(move |id: String| {
            selected_id.set(Some(id.to_string()));
            selected_data.set(get_data_for_id(id));
        })
    };

    let rows = users.iter().map(|agg| {
        let year_month = agg.year_month.clone();
        let on_row_click = {
            let selected_id = selected_id.clone();
            let selected_data = selected_data.clone();
            Callback::from(move |_| {
                selected_id.set(Some(year_month.clone()));
                selected_data.set(get_data_for_id(year_month.clone()));
            })
        };
        html! {
            <tr
                class="border-b hover:bg-gray-100 relative"
                onclick={on_row_click}
            >
                <td class="px-4 py-2">{ &agg.year_month }</td>
                <td class={classes!(
                    "px-4",
                    "py-2",
                    if agg.sum < 0.0 { "text-red-700" } else { "text-green-700" },
                )} style="text-align: right">{ '€' }{ format!("{:.02}", agg.sum.abs()) }</td>
            </tr>
        }
    });

    html! {
        <>
        <div class="p-4 max-h-screen overflow-auto">
            <div class="min-w-full overflow-x-auto border rounded shadow-md">
                <table class="min-w-full table-auto bg-white">
                    <thead class="bg-gray-200 sticky top-0 z-10">
                        <tr>
                            <th class="px-4 py-2 text-left">{ "Value" }</th>
                            <th class="px-4 py-2 text-left">{ "Name" }</th>
                        </tr>
                    </thead>
                    <tbody>
                        { for rows }
                    </tbody>
                </table>
            </div>
        </div>
        <div style="display: flex;">
            <div style="flex: 1;">
                <button onclick={Callback::from(move |_| on_row_click.emit("2".to_string()))}>
                    { "Click Main Row 2" }
                </button>
            </div>
            <InfoPanel selected_data={(*selected_data).clone()} />
        </div>
        </>
    }
}

// Mock main row data structure
#[derive(Clone, PartialEq)]
pub struct RowData {
    pub id: String,
    pub name: String,
    pub sub_items: Vec<SubItem>,
}

#[derive(Clone, PartialEq)]
pub struct SubItem {
    pub id: String,
    pub label: String,
}

fn get_data_for_id(id: String) -> RowData {
    RowData {
        id: id.clone(),
        name: format!("Main Row {}", id.clone()),
        sub_items: vec![
            SubItem { id: "1".to_string(), label: format!("Item {}.A", id) },
            SubItem { id: "2".to_string(), label: format!("Item {}.B", id) },
        ],
    }
}
