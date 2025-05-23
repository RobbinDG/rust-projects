use crate::aggregates_table::AggregatesTable;
use crate::upload_transactions::UploadTransactions;
use yew::prelude::*;

#[function_component(OverviewPage)]
pub fn overview_page() -> Html {
    html! {
        <div class="h-[calc(100%-2rem)] m-4 flex flex-row gap-5 w-full">
            <div class="container w-xs h-fit">
                <UploadTransactions />
            </div>
            <AggregatesTable />
        </div>
    }
}
