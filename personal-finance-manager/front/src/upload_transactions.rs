use yew::{function_component, html, Html};

#[function_component(UploadTransactions)]
pub fn upload_transactions() -> Html {
    html!(
        <form action="http://127.0.0.1:8000/transactions" enctype="multipart/form-data" method="POST" class="max-w-sm mx-auto" >
            <input type="file" id="myFile" name="filename" accept=".tsv,.csv" class="block w-full text-sm text-gray-900 border border-gray-300 rounded-lg cursor-pointer bg-gray-50 dark:text-gray-400 focus:outline-none dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400" />
            <input type="submit" class="text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:ring-blue-300 font-medium rounded-lg text-sm px-5 py-2.5 me-2 mb-2 dark:bg-blue-600 dark:hover:bg-blue-700 focus:outline-none dark:focus:ring-blue-800" />
        </form>
    )
}
