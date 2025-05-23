use crate::aggregates_table::RowData;
use crate::change_category::ChangeCategory;
use crate::editable_table::EditableTable;
use crate::transactions_page::TransactionWithCategory;
use crate::transactions_table::TransactionsTable;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct InfoPanelProps {
    pub selected_data: Option<RowData>,
    pub transactions: Vec<TransactionWithCategory>,
}

#[function_component(InfoPanel)]
pub fn info_panel(props: &InfoPanelProps) -> Html {
    let selected_row = use_state(|| None::<(String, Option<String>)>);
    let (title, rows) = match &props.selected_data {
        None => ("No selection".to_owned(), Vec::new()),
        Some(data) => (
            format!("Details for {}", data.year_month),
            data.items.clone(),
        ),
    };

    let on_row_click = {
        let selected_row = selected_row.clone();
        Callback::from(move |t: TransactionWithCategory| {
            selected_row.set(Some((t.name_other, t.category)));
        })
    };

    html! {
        <div class="flex flex-col space-y-5 h-full max-h-full w-full">
            <div class="container">
                <h3>{ title }</h3>
                <ChangeCategory name={(*selected_row).clone()} />
            </div>
            <div class="flex-1 container overflow-y-auto">
                <TransactionsTable transactions={props.transactions.clone()} on_row_click={Some(on_row_click)}/>
            </div>
        </div>
    }
}
