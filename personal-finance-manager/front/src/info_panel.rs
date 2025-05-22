use yew::prelude::*;
use crate::aggregates_table::RowData;
use crate::editable_table::EditableTable;

#[derive(Properties, PartialEq)]
pub struct InfoPanelProps {
    pub selected_data: RowData,
}

#[function_component(InfoPanel)]
pub fn info_panel(props: &InfoPanelProps) -> Html {
    html! {
        <div style="width: 300px; padding: 1rem; background: #f5f5f5;">
            <h3>{ format!("Details for {}", props.selected_data.name) }</h3>
            <input type="text" placeholder="Free text..." />
            <EditableTable sub_items={props.selected_data.sub_items.clone()} />
        </div>
    }
}
