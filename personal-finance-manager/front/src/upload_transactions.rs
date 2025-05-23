use yew::{function_component, html, Html};

#[function_component(UploadTransactions)]
pub fn upload_transactions() -> Html {
    html!(
        <form action="http://127.0.0.1:8000/transactions" enctype="multipart/form-data" method="POST" >
            <input type="file" id="myFile" name="filename" accept=".tsv,.csv" />
            <input type="submit" />
        </form>
    )
}
