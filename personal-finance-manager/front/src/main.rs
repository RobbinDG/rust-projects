mod transactions_table;
mod aggregates_table;
mod upload_transactions;
mod routes;
mod editable_table;
mod editable_row;
mod info_panel;
mod row_editor;
mod transactions_page;
mod change_category;
mod overview_page;

use crate::routes::{switch, AppRoute};
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <BrowserRouter>
            <nav class="flex space-x-4 p-4">
                    <Link<AppRoute> to={AppRoute::Transactions}>{ "Transactions" }</Link<AppRoute>>
                    <Link<AppRoute> to={AppRoute::Overview}>{ "Overview" }</Link<AppRoute>>
            </nav>
            <Switch<AppRoute> render={switch} />
        </BrowserRouter>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
