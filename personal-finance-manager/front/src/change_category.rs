use crate::aggregates_table::API_URL;
use gloo::net::http::Request;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ChangeCategoryProps {
    pub name: Option<(String, Option<String>)>,
}

#[function_component(ChangeCategory)]
pub fn change_category(props: &ChangeCategoryProps) -> Html {
    let current = match &props.name {
        Some((_, Some(c))) => c.clone(),
        _ => "".to_owned(),
    };
    let input_value = use_state(|| current);

     {
        let input_value = input_value.clone();
        let name = props.name.clone();
        use_effect_with(
            name,
            move |name| {
                let current = match name {
                    Some((_, Some(c))) => c.clone(),
                    _ => "".to_owned()
                };
                input_value.set(current); // reset text field
                || ()
            },
        );
    }

    let on_input = {
        let input_value = input_value.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                input_value.set(input.value());
            }
        })
    };

    let on_click = {
        let name = props.name.clone();
        let value = (*input_value).clone();
        Callback::from(move |_| {
            if let Some((name, _)) = &name {
                let name = name.clone();
                let value = value.clone();

                wasm_bindgen_futures::spawn_local(async move {
                    let endpoint = format!("{}/categories/{}/{}", API_URL, value, name);
                    Request::post(&endpoint).send().await.unwrap();
                });
            }
        })
    };

    let name = match &props.name {
        Some((n, _)) => n.clone(),
        _ => "None selected".to_owned(),
    };

    html! {
        <>
        <div class="space-y-2">
            <p>{ name }</p>
            <input
                type="text"
                class="w-full p-2 border rounded"
                value={(*input_value).clone()}
                oninput={on_input}
            />
            <button
                class="bg-blue-500 hover:bg-blue-600 text-white px-4 py-2 rounded"
                onclick={on_click}
            >
                { "Submit" }
            </button>
        </div>
        </>
    }
}
