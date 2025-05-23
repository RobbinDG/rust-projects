use yew::prelude::*;
use crate::editable_row::EditableRow;
use crate::aggregates_table::SubItem;

#[derive(Properties, PartialEq)]
pub struct EditableTableProps {
    pub sub_items: Vec<SubItem>,
}

#[function_component(EditableTable)]
pub fn editable_table(props: &EditableTableProps) -> Html {
    html! {
        <div class="table-container">
            <table class="w-full">
                <thead>
                    <tr>
                        <th class="text-left">{ "Category" }</th>
                        <th class="text-right">{ "Value" }</th>
                    </tr>
                </thead>
                <tbody>
                    {
                        props.sub_items.iter().map(|item| {
                            html! {
                                <EditableRow key={item.category.clone()} item={item.clone()} />
                            }
                        }).collect::<Html>()
                    }
                </tbody>
            </table>
        </div>
    }
}
