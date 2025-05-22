use yew::prelude::*;
use crate::aggregates_table::SubItem;

#[derive(Properties, PartialEq)]
pub struct RowEditorProps {
    pub item: SubItem,
}

#[function_component(RowEditor)]
pub fn row_editor(props: &RowEditorProps) -> Html {
    let input = use_state(|| props.item.label.clone());

    let oninput = {
        let input = input.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(input_el) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                input.set(input_el.value());
            }
        })
    };

    let on_submit = {
        let value = (*input).clone();
        Callback::from(move |_| {
            web_sys::console::log_1(&format!("Updated '{}'", value).into());
        })
    };

    html! {
        <div>
            <input type="text" value={(*input).clone()} oninput={oninput} />
            <button onclick={on_submit}>{ "Submit" }</button>
        </div>
    }
}
