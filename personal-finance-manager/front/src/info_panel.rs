use yew::prelude::*;
use crate::aggregates_table::RowData;
use crate::editable_table::EditableTable;

#[derive(Properties, PartialEq)]
pub struct InfoPanelProps {
    pub selected_data: Option<RowData>,
}

#[function_component(InfoPanel)]
pub fn info_panel(props: &InfoPanelProps) -> Html {
    let (title, rows) = match &props.selected_data {
        None =>
            ("No selection".to_owned(), Vec::new()),
        Some(data) => (
            format!("Details for {}", data.year_month), data.items.clone()
        )
    };
    html! {
        <div style="width: 300px; padding: 1rem; background: #f5f5f5;">
            <h3>{ title }</h3>
            <input type="text" placeholder="Free text..." />
            <EditableTable sub_items={rows} />
        </div>
    }
}
