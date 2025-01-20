use crate::new::queue_store::QueueStore;
use crate::new::request_worker::RequestWorker;
use backend::protocol::new::codec::{encode, CodecError};
use backend::protocol::new::request_error::RequestError;
use backend::protocol::request::{
    CheckQueue, CreateQueue, DeleteQueue, GetProperties, ListQueues, SupportedRequest,
};
use backend::protocol::Request;
use backend::stream_io::{StreamIO, StreamIOError};
use log::error;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

trait StreamResponder<R>
where
    R: Request,
{
    async fn send_over_stream(self, stream: &mut StreamIO) -> std::io::Result<()>;
}

impl<R> StreamResponder<R> for Result<R::Response, RequestError>
where
    R: Request,
{
    async fn send_over_stream(self, stream: &mut StreamIO) -> std::io::Result<()> {
        stream.write_encode(&self).await.or_else(|e| match e {
            StreamIOError::Stream(s) => Err(s),
            StreamIOError::Codec(_) => {
                error!("Response poorly formatted");
                Ok(())
            }
        })
    }
}

trait Handler<R>
where
    R: Request,
{
    fn handle(&mut self, request: &R) -> Result<R::Response, RequestError>;
}

struct ListQueuesHandler {
    queues: Arc<Mutex<QueueStore>>,
}

impl Handler<ListQueues> for ListQueuesHandler {
    fn handle(
        &mut self,
        request: &ListQueues,
    ) -> Result<<ListQueues as Request>::Response, RequestError> {
        Ok(self.queues.lock()?.list())
    }
}

struct CheckQueueHandler {}

impl Handler<CheckQueue> for CheckQueueHandler {
    fn handle(
        &mut self,
        request: &CheckQueue,
    ) -> Result<<CheckQueue as Request>::Response, RequestError> {
        todo!()
    }
}

struct CreateQueueHandler {}

impl Handler<CreateQueue> for CreateQueueHandler {
    fn handle(
        &mut self,
        request: &CreateQueue,
    ) -> Result<<CreateQueue as Request>::Response, RequestError> {
        todo!()
    }
}

struct DeleteQueueHandler {}

impl Handler<DeleteQueue> for DeleteQueueHandler {
    fn handle(
        &mut self,
        request: &DeleteQueue,
    ) -> Result<<DeleteQueue as Request>::Response, RequestError> {
        todo!()
    }
}

struct GetPropertiesHandler {}

impl Handler<GetProperties> for GetPropertiesHandler {
    fn handle(
        &mut self,
        request: &GetProperties,
    ) -> Result<<GetProperties as Request>::Response, RequestError> {
        todo!()
    }
}

pub struct RequestDispatcher {
    list_queues: ListQueuesHandler,
}

impl RequestDispatcher {
    pub fn new(queue_store: Arc<Mutex<QueueStore>>) -> Self {
        Self {
            list_queues: ListQueuesHandler {
                queues: queue_store,
            },
        }
    }

    pub async fn dispatch(&mut self, request: SupportedRequest) -> Result<Vec<u8>, CodecError> {
        match request {
            SupportedRequest::ListQueues(r) => handle_and_send(r, &mut self.list_queues),
            SupportedRequest::CheckQueue(r) => handle_and_send(r, &mut CheckQueueHandler {}),
            SupportedRequest::CreateQueue(r) => handle_and_send(r, &mut CreateQueueHandler {}),
            SupportedRequest::DeleteQueue(r) => handle_and_send(r, &mut DeleteQueueHandler {}),
            SupportedRequest::GetProperties(r) => handle_and_send(r, &mut GetPropertiesHandler {}),
        }
    }
}

fn handle_and_send<R, H>(request: R, handler: &mut H) -> Result<Vec<u8>, CodecError>
where
    R: Request,
    H: Handler<R>,
{
    let x: Result<R, RequestError> = Ok(request);
    let y = x.and_then(|lq| {
        handler
            .handle(&lq)
            .map_err(|_| RequestError::RequestHandlingError)
    });
    encode(&y)
}
