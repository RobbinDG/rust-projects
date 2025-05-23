use yew::{function_component, html, Html};

#[function_component(UploadTransactions)]
pub fn upload_transactions() -> Html {
    html!(
        <form action="http://127.0.0.1:8000/transactions" enctype="multipart/form-data" method="POST" class="max-w-sm mx-auto" >
            <input type="file" id="myFile" name="filename" accept=".tsv,.csv" class="block w-full text-sm text-gray-900 border border-gray-300 rounded-lg cursor-pointer bg-gray-50 dark:text-gray-400 focus:outline-none dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400" />
            <input type="submit" />
        </form>
    )
}
