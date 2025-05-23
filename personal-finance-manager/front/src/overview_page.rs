use yew::prelude::*;
use crate::aggregates_table::AggregatesTable;
use crate::info_panel::InfoPanel;
use crate::upload_transactions::UploadTransactions;

#[function_component(OverviewPage)]
pub fn overview_page() -> Html {
    html! {
        <>
            <UploadTransactions />
            <AggregatesTable />
        </>

    }
}