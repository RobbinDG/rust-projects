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
            <div class="flex flex-col h-screen w-full">
                <nav class="top-0 w-full bg-gray-800">
                  <div class="mx-auto max-w-7xl px-2 sm:px-6 lg:px-8">
                    <div class="relative flex h-16 items-center justify-between">
                      <div class="flex flex-1 items-center justify-center sm:items-stretch sm:justify-start">
                        <div class="hidden sm:ml-6 sm:block">
                          <div class="flex space-x-4">
                            <Link<AppRoute> to={AppRoute::Transactions}><p class="rounded-md px-3 py-2 text-sm font-medium text-gray-300 hover:bg-gray-700 hover:text-white">{ "Transactions" }</p></Link<AppRoute>>
                            <Link<AppRoute> to={AppRoute::Overview}><p class="rounded-md px-3 py-2 text-sm font-medium text-gray-300 hover:bg-gray-700 hover:text-white">{ "Overview" }</p></Link<AppRoute>>
                          </div>
                        </div>
                      </div>
                    </div>
                  </div>
                </nav>
                <div class="h-[calc(100vh-64px)]">
                    <Switch<AppRoute> render={switch} />
                </div>
            </div>
        </BrowserRouter>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
