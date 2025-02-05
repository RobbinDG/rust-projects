use crate::server_connector::ServerConnector;
use backend::protocol::request::SupportedRequest;
use backend::protocol::Request;
use iced::Task;
use std::sync::Arc;
use iced::advanced::graphics::futures::MaybeSend;
use tokio::sync::Mutex;

pub fn request_task<R, M, F>(
    connector: Arc<Mutex<ServerConnector>>,
    request: R,
    response_handler: F,
) -> Task<Result<M, ()>>
where
    R: Request + 'static,
    F: Fn(<R as Request>::Response) -> M + MaybeSend + Sync + 'static,
    M: Send + 'static,
    SupportedRequest: From<R>,
{
    Task::perform(
        async move {
            match connector.lock().await.client().await {
                Ok(client) => client.transfer_admin_request(request).await.ok(),
                Err(_) => None,
            }
        },
        move |result| match result {
            None => Err(()),
            Some(response) => Ok(response_handler(response)),
        },
    )
}
