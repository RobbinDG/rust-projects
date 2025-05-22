use crate::aggregates_table::SubItem;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct EditableRowProps {
    pub item: SubItem,
}

#[function_component(EditableRow)]
pub fn editable_row(props: &EditableRowProps) -> Html {
    html! {
        <>
            <tr style="cursor: pointer;">
                <td>{ &props.item.category }</td>
                <td>{ format!("{:.02}", &props.item.breakdown_value) }</td>
            </tr>
        </>
    }
}
