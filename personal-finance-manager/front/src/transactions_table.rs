use crate::transactions_page::TransactionWithCategory;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct TransactionTableProps {
    pub transactions: Vec<TransactionWithCategory>,
    #[prop_or_default]
    pub on_row_click: Option<Callback<TransactionWithCategory>>,
}

#[function_component(TransactionsTable)]
pub fn transactions_table(props: &TransactionTableProps) -> Html {
    let hovered_row = use_state(|| None);

    let tooltip = {
        if let Some((id, x, y)) = *hovered_row {
            if let Some(row) = props.transactions.iter().find(|r| r.MTCN == id) {
                let iban_other = match &row.IBAN_other {
                    Some(d) => {
                        if d.is_empty() {
                            "Unknown"
                        } else {
                            d
                        }
                    }
                    None => "Unknown",
                };
                let desc = match &row.description {
                    Some(d) => {
                        if d.is_empty() {
                            "No Description"
                        } else {
                            d
                        }
                    }
                    None => "No Description",
                };
                html! {
                    <div
                        class="fixed z-50 bg-white text-sm text-gray-800 border rounded shadow-lg p-2"
                        style={format!("left: {}px; top: {}px;", x + 10, y + 10)}
                    >
                        <div class="font-semibold">{"Details"}</div>
                        <div>{"From: "}{ iban_other }</div>
                        <div>{"Description: "}{ desc }</div>
                        <div>{"Balance after: €"}{ &row.balance_after }</div>
                    </div>
                }
            } else {
                html! {}
            }
        } else {
            html! {}
        }
    };

    let rows = props.transactions.iter().map(|transaction| {
        let hovered_row_clone = hovered_row.clone();
        let id = transaction.MTCN;
        let on_mouse_enter = Callback::from(move |e: MouseEvent| {
            hovered_row_clone.set(Some((id, e.client_x(), e.client_y())));
        });

        let hovered_row = hovered_row.clone();
        let on_mouse_leave = Callback::from(move |_| {
            hovered_row.set(None);
        });
        let on_click = props.on_row_click.clone();
        let transaction_clone = transaction.clone();

        html! {
            <tr
                class="border-b hover:bg-gray-100 relative"
                onmouseenter={on_mouse_enter}
                onmouseleave={on_mouse_leave}
                onclick={Callback::from(move |_| {
                    if let Some(cb) = &on_click {
                        cb.emit(transaction_clone.clone());
                    }
                })}
            >
                <td class={classes!(
                    "px-4",
                    "py-2",
                    if transaction.value < 0.0 { "text-red-700" } else { "text-green-700" },
                )} style="text-align: right">{ '€' }{ format!("{:.02}", transaction.value.abs()) }</td>
                <td class="px-4 py-2">{ &transaction.name_other.clone() }</td>
                <td class="px-4 py-2">{ &transaction.category.clone() }</td>
            </tr>
        }
    });
    html! {
        <>
        { tooltip }
        <div class="flex-1 overflow-y-auto">
            <table class="table-auto bg-white">
                <thead class="bg-gray-200 sticky top-0 z-10">
                    <tr>
                        <th class="px-4 py-2 text-left">{ "Value" }</th>
                        <th class="px-4 py-2 text-left">{ "Name" }</th>
                        <th class="px-4 py-2 text-left">{ "Category" }</th>
                    </tr>
                </thead>
                <tbody>
                    { for rows }
                </tbody>
            </table>
        </div>
        </>
    }
}
