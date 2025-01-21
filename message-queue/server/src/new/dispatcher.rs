use crate::new::queue_store::QueueStore;
use backend::protocol::new::codec::encode;
use backend::protocol::new::request_error::RequestError;
use backend::protocol::request::SupportedRequest;
use backend::protocol::Request;
use std::sync::{Arc, Mutex};
use crate::new::request_handler::{CheckQueueHandler, CreateQueueHandler, DeleteQueueHandler, GetPropertiesHandler, Handler, ListQueuesHandler};

pub struct RequestDispatcher {
    list_queues: ListQueuesHandler,
    check_queue: CheckQueueHandler,
    create: CreateQueueHandler,
    delete: DeleteQueueHandler,
    get_props: GetPropertiesHandler,
}

impl RequestDispatcher {
    pub fn new(queue_store: Arc<Mutex<QueueStore>>) -> Self {
        Self {
            list_queues: ListQueuesHandler::new(queue_store.clone()),
            check_queue: CheckQueueHandler::new(queue_store.clone()),
            create: CreateQueueHandler::new(queue_store.clone()),
            delete: DeleteQueueHandler::new(queue_store.clone()),
            get_props: GetPropertiesHandler::new(queue_store.clone()),
        }
    }

    pub async fn dispatch(&mut self, request: SupportedRequest) -> Result<Vec<u8>, RequestError> {
        match request {
            SupportedRequest::ListQueues(r) => handle_and_encode(r, &mut self.list_queues),
            SupportedRequest::CheckQueue(r) => handle_and_encode(r, &mut self.check_queue),
            SupportedRequest::CreateQueue(r) => handle_and_encode(r, &mut self.create),
            SupportedRequest::DeleteQueue(r) => handle_and_encode(r, &mut self.delete),
            SupportedRequest::GetProperties(r) => handle_and_encode(r, &mut self.get_props),
        }
    }
}

fn handle_and_encode<R, H>(request: R, handler: &mut H) -> Result<Vec<u8>, RequestError>
where
    R: Request,
    H: Handler<R>,
{
    let x: Result<R, RequestError> = Ok(request);
    x.and_then(|lq| {
        handler
            .handle(&lq)
            .map_err(|_| RequestError::RequestHandlingError)
    })
    .and_then(|response| encode(&response).or(Err(RequestError::PayloadEncodeError)))
}
