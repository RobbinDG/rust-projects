use gloo_net::http::Request;
use serde::Deserialize;
use yew::prelude::*;

const API_URL: &str = "http://127.0.0.1:8000";

#[derive(Deserialize)]
struct Transaction {
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
}

#[function_component(App)]
pub fn app() -> Html {
    let response = use_state(|| String::new());
    let loading = use_state(|| false);

    let onclick = {
        let response = response.clone();
        let loading = loading.clone();

        Callback::from(move |_| {
            let response = response.clone();
            let loading = loading.clone();
            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);
                match Request::get((API_URL.to_owned() + "/hello/robbin/25").as_str())
                    .send()
                    .await
                {
                    Ok(resp) => {
                        if let Ok(text) = resp.text().await {
                            response.set(text);
                        } else {
                            response.set("Failed to read response body".into());
                        }
                    }
                    Err(e) => {
                        response.set(format!("Request failed {}", e));
                    }
                }
                loading.set(false);
            });
        })
    };

    let users = use_state(|| Vec::<Transaction>::new());
    let users_clone = users.clone();

    use_effect_with((), move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            let fetched: Vec<Transaction> =
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

    html! {
        <>
        <div>
            <button {onclick} disabled={*loading}>{ if *loading { "Loading..." } else { "Fetch Data" } }</button>
            <p>{ (*response).clone() }</p>
        </div>
        <div class="p-4 max-h-screen overflow-auto">
            <div class="min-w-full overflow-x-auto border rounded shadow-md">
                <table class="min-w-full table-auto bg-white">
                    <thead class="bg-gray-200 sticky top-0 z-10">
                        <tr>
                            <th class="px-4 py-2 text-left">{ "ID" }</th>
                            <th class="px-4 py-2 text-left">{ "Name" }</th>
                            <th class="px-4 py-2 text-left">{ "Email" }</th>
                        </tr>
                    </thead>
                    <tbody>
                        {
                            for users.iter().map(|transaction| html! {
                                <tr key={ transaction.MTCN}  class="border-b hover:bg-gray-100">
                                    <td class={classes!(
                                        "px-4",
                                        "py-2",
                                        if transaction.value < 0.0 { "text-red-700" } else { "text-green-700" }
                                    )}>{ '€' }{ transaction.value.abs() }</td>
                                    <td class="px-4 py-2">{ '€' }{ transaction.balance_after.abs() }</td>
                                    <td class="px-4 py-2">{ &transaction.name_other }</td>
                                    <td class="px-4 py-2">{ &transaction.description }</td>
                                </tr>
                            })
                        }
                    </tbody>
                </table>
            </div>
        </div>
        </>
    }
}
