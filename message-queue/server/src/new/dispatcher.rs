use crate::new::request_worker::RequestWorker;
use backend::protocol::new::codec::{encode, CodecError};
use backend::protocol::new::request_error::RequestError;
use backend::protocol::request::{CheckQueue, CreateQueue, DeleteQueue, GetProperties, ListQueues, SupportedRequest};
use backend::protocol::{Request, ResponseError};
use backend::stream_io::{StreamIO, StreamIOError};
use log::error;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

trait StreamResponder<R>
where
    R: Request,
{
    async fn send_over_stream(self, stream: &mut StreamIO) -> std::io::Result<()>;
}

impl<R> StreamResponder<R> for Result<R::Response, ResponseError>
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
    fn handle(&mut self, request: &R) -> Result<R::Response, ResponseError>;
}

struct ListQueuesHandler {}

impl Handler<ListQueues> for ListQueuesHandler {
    fn handle(
        &mut self,
        request: &ListQueues,
    ) -> Result<<ListQueues as Request>::Response, ResponseError> {
        todo!()
    }
}

struct CheckQueueHandler {}

impl Handler<CheckQueue> for CheckQueueHandler {
    fn handle(
        &mut self,
        request: &CheckQueue,
    ) -> Result<<CheckQueue as Request>::Response, ResponseError> {
        todo!()
    }
}

struct CreateQueueHandler {}

impl Handler<CreateQueue> for CreateQueueHandler {
    fn handle(&mut self, request: &CreateQueue) -> Result<<CreateQueue as Request>::Response, ResponseError> {
        todo!()
    }
}

struct DeleteQueueHandler {}

impl Handler<DeleteQueue> for DeleteQueueHandler {
    fn handle(&mut self, request: &DeleteQueue) -> Result<<DeleteQueue as Request>::Response, ResponseError> {
        todo!()
    }
}

struct GetPropertiesHandler {}

impl Handler<GetProperties> for GetPropertiesHandler {
    fn handle(&mut self, request: &GetProperties) -> Result<<GetProperties as Request>::Response, ResponseError> {
        todo!()
    }
}

pub struct RequestDispatcher {}

impl RequestDispatcher {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn dispatch(&self, request: SupportedRequest) -> Result<Vec<u8>, CodecError> {
        match request {
            SupportedRequest::ListQueues(r) => handle_and_send(r, &mut ListQueuesHandler {}),
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
