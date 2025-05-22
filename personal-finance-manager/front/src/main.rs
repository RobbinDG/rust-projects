mod transactions_table;
mod aggregates_table;
mod upload_transactions;
mod routes;
mod editable_table;
mod editable_row;
mod info_panel;
mod row_editor;
mod transactions_page;

use crate::routes::{switch, AppRoute};
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <BrowserRouter>
            <nav>
                <ul>
                    <li><Link<AppRoute> to={AppRoute::Transactions}>{ "Transactions" }</Link<AppRoute>></li>
                    <li><Link<AppRoute> to={AppRoute::Overview}>{ "Overview" }</Link<AppRoute>></li>
                </ul>
            </nav>
            <Switch<AppRoute> render={switch} />
        </BrowserRouter>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
