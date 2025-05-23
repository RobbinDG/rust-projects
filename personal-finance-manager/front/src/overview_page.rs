use crate::aggregates_table::AggregatesTable;
use crate::upload_transactions::UploadTransactions;
use yew::prelude::*;

#[function_component(OverviewPage)]
pub fn overview_page() -> Html {
    html! {
        <div class="h-[calc(100vh-2rem)] m-4 flex flex-row gap-5 max-h-screen max-w-full">
            <UploadTransactions />
            <AggregatesTable />
        </div>
    }
}
