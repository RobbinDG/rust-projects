use gloo_net::http::Request;
use serde::Deserialize;
use yew::prelude::*;

const API_URL: &str = "http://127.0.0.1:8000";

#[derive(Deserialize)]
struct MonthAggregate {
    month_year: String,
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

    let rows = users.iter().map(|agg| {
        html! {
            <tr
                class="border-b hover:bg-gray-100 relative"
            >
                <td class="px-4 py-2">{ &agg.month_year }</td>
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
        </>
    }
}
