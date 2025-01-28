use crate::queue_store::QueueStore;
use crate::request_handler::{CheckQueueHandler, CreateQueueHandler, DeleteQueueHandler, GetPropertiesHandler, GetTopicBreakdownHandler, Handler, ListQueuesHandler, PublishHandler, ReceiveHandler, SubscribeHandler};
use crate::router::Router;
use crate::subscription_manager::SubscriptionManager;
use backend::protocol::client_id::ClientID;
use backend::protocol::codec::encode;
use backend::protocol::request::{GetTopicBreakdown, SupportedRequest};
use backend::protocol::request_error::RequestError;
use backend::protocol::Request;
use std::sync::{Arc, Mutex};

/// A helper object to dispatch requests to a designated handler and encode their responses.
pub struct RequestDispatcher {
    list_queues: ListQueuesHandler,
    check_queue: CheckQueueHandler,
    create: CreateQueueHandler,
    delete: DeleteQueueHandler,
    get_props: GetPropertiesHandler,
    publish: PublishHandler,
    subscribe: SubscribeHandler,
    receive: ReceiveHandler,
    get_topic_breakdown: GetTopicBreakdownHandler,
}

impl RequestDispatcher {
    /// Initialise a `RequestDispatcher`.
    ///
    /// # Arguments
    ///
    /// * `queue_store`: a shared reference to the queue store to modify by executing the
    ///     requests that are dispatched using this dispatcher.
    ///
    /// returns: `RequestDispatcher`
    pub fn new(queue_store: Arc<Mutex<QueueStore>>) -> Self {
        let router = Arc::new(Mutex::new(Router::new(queue_store.clone())));
        let subscription_manager =
            Arc::new(Mutex::new(SubscriptionManager::new(queue_store.clone())));
        Self {
            list_queues: ListQueuesHandler::new(queue_store.clone()),
            check_queue: CheckQueueHandler::new(queue_store.clone()),
            create: CreateQueueHandler::new(queue_store.clone()),
            delete: DeleteQueueHandler::new(queue_store.clone()),
            get_props: GetPropertiesHandler::new(queue_store.clone()),
            publish: PublishHandler::new(router.clone()),
            subscribe: SubscribeHandler::new(subscription_manager.clone()),
            receive: ReceiveHandler::new(subscription_manager, router),
            get_topic_breakdown: GetTopicBreakdownHandler::new(queue_store),
        }
    }

    /// Dispatch a supported request to the handler and return the encoded response (or error).
    ///
    /// # Arguments
    ///
    /// * `request`: a `SupportedRequest` that is to be dispatched.
    ///
    /// returns: `Result<Vec<u8, Global>, RequestError>` The byte-encoded result or a request
    ///     error.
    pub async fn dispatch(
        &mut self,
        request: SupportedRequest,
        client: ClientID,
    ) -> Result<Vec<u8>, RequestError> {
        match request {
            SupportedRequest::ListQueues(r) => handle_and_encode(r, &mut self.list_queues, client),
            SupportedRequest::CheckQueue(r) => handle_and_encode(r, &mut self.check_queue, client),
            SupportedRequest::CreateQueue(r) => handle_and_encode(r, &mut self.create, client),
            SupportedRequest::DeleteQueue(r) => handle_and_encode(r, &mut self.delete, client),
            SupportedRequest::GetProperties(r) => handle_and_encode(r, &mut self.get_props, client),
            SupportedRequest::Publish(r) => handle_and_encode(r, &mut self.publish, client),
            SupportedRequest::Subscribe(r) => handle_and_encode(r, &mut self.subscribe, client),
            SupportedRequest::Receive(r) => handle_and_encode(r, &mut self.receive, client),
            SupportedRequest::GetTopicBreakdown(r) => handle_and_encode(r, &mut self.get_topic_breakdown, client),
        }
    }
}

/// A generic helper method to use a `Handler` instance to handle a `Request` and encode
/// the response for a normalised output independent of generics.
///
/// # Arguments
///
/// * `request`: an implementor of `Request`; the request to handle.
/// * `handler`: a corresponding `Handler` that can have `handle` called on `request`.
///
/// returns: `Result<Vec<u8, Global>, RequestError>` the encoded result or an error.
fn handle_and_encode<R, H>(
    request: R,
    handler: &mut H,
    client: ClientID,
) -> Result<Vec<u8>, RequestError>
where
    R: Request,
    H: Handler<R>,
{
    let x: Result<R, RequestError> = Ok(request);
    x.and_then(|lq| {
        handler
            .handle(lq, client)
            .map_err(|_| RequestError::RequestHandlingError)
    })
    .and_then(|response| encode(&response).or(Err(RequestError::PayloadEncodeError)))
}
