use gloo_net::http::Request;
use yew::prelude::*;

const API_URL: &str = "http://127.0.0.1:8000";

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
                match Request::get((API_URL.to_owned() + "/hello/robbin/25").as_str()).send().await {
                    Ok(resp) => {
                        if let Ok(text) = resp.text().await  {
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

    html! {
        <div>
            <button {onclick} disabled={*loading}>{ if *loading { "Loading..." } else { "Fetch Data" } }</button>
            <p>{ (*response).clone() }</p>
        </div>
    }
}
