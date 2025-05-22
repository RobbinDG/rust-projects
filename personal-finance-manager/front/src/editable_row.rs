use yew::prelude::*;
use crate::aggregates_table::SubItem;
use crate::row_editor::RowEditor;

#[derive(Properties, PartialEq)]
pub struct EditableRowProps {
    pub item: SubItem,
}

#[function_component(EditableRow)]
pub fn editable_row(props: &EditableRowProps) -> Html {
    let show_editor = use_state(|| false);

    let toggle_editor = {
        let show_editor = show_editor.clone();
        Callback::from(move |_| show_editor.set(!*show_editor))
    };

    html! {
        <>
            <tr onclick={toggle_editor} style="cursor: pointer;">
                <td>{ &props.item.category }</td>
                <td>{ format!("{:.02}", &props.item.breakdown_value) }</td>
            </tr>
            {
                if *show_editor {
                    html! {
                        <tr>
                            <td colspan="1">
                                <RowEditor item={props.item.clone()} />
                            </td>
                        </tr>
                    }
                } else {
                    html! {}
                }
            }
        </>
    }
}
