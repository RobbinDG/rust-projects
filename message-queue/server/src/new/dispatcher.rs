use crate::new::queue_store::QueueStore;
use crate::new::request_handler::{CheckQueueHandler, CreateQueueHandler, DeleteQueueHandler, GetPropertiesHandler, Handler, ListQueuesHandler, PublishHandler, ReceiveHandler};
use backend::protocol::new::codec::encode;
use backend::protocol::new::request_error::RequestError;
use backend::protocol::request::SupportedRequest;
use backend::protocol::Request;
use std::sync::{Arc, Mutex};
use crate::new::router::Router;

/// A helper object to dispatch requests to a designated handler and encode their responses.
pub struct RequestDispatcher {
    router: Arc<Mutex<Router>>,
    list_queues: ListQueuesHandler,
    check_queue: CheckQueueHandler,
    create: CreateQueueHandler,
    delete: DeleteQueueHandler,
    get_props: GetPropertiesHandler,
    publish: PublishHandler,
    receive: ReceiveHandler,
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
        Self {
            router: router.clone(),
            list_queues: ListQueuesHandler::new(queue_store.clone()),
            check_queue: CheckQueueHandler::new(queue_store.clone()),
            create: CreateQueueHandler::new(queue_store.clone()),
            delete: DeleteQueueHandler::new(queue_store.clone()),
            get_props: GetPropertiesHandler::new(queue_store.clone()),
            publish: PublishHandler::new(router.clone()),
            receive: ReceiveHandler::new(router),
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
    pub async fn dispatch(&mut self, request: SupportedRequest) -> Result<Vec<u8>, RequestError> {
        match request {
            SupportedRequest::ListQueues(r) => handle_and_encode(r, &mut self.list_queues),
            SupportedRequest::CheckQueue(r) => handle_and_encode(r, &mut self.check_queue),
            SupportedRequest::CreateQueue(r) => handle_and_encode(r, &mut self.create),
            SupportedRequest::DeleteQueue(r) => handle_and_encode(r, &mut self.delete),
            SupportedRequest::GetProperties(r) => handle_and_encode(r, &mut self.get_props),
            SupportedRequest::Publish(r) => handle_and_encode(r, &mut self.publish),
            SupportedRequest::Receive(r) => handle_and_encode(r, &mut self.receive),
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
fn handle_and_encode<R, H>(request: R, handler: &mut H) -> Result<Vec<u8>, RequestError>
where
    R: Request,
    H: Handler<R>,
{
    let x: Result<R, RequestError> = Ok(request);
    x.and_then(|lq| {
        handler
            .handle(lq)
            .map_err(|_| RequestError::RequestHandlingError)
    })
    .and_then(|response| encode(&response).or(Err(RequestError::PayloadEncodeError)))
}
