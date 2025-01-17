use crate::new::request_worker::RequestWorker;
use crate::request_handler::RequestHandler;
use crate::server_error::ServerError;
use backend::protocol::new::codec::{encode, CodecError};
use backend::protocol::new::request_error::RequestError;
use backend::protocol::request::{CheckQueue, CreateQueue, ListQueues, SupportedRequest};
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

pub struct RequestDispatcher {}

impl RequestDispatcher {
    pub async fn dispatch<R>(
        &self,
        request: Result<SupportedRequest, RequestError>,
    ) -> Result<Vec<u8>, CodecError>
    where
        R: Request,
        SupportedRequest: From<R>,
    {
        match request {
            Ok(SupportedRequest::ListQueues(r)) => handle_and_send(r, &mut ListQueuesHandler {}),
            Ok(SupportedRequest::CheckQueue(r)) => handle_and_send(r, &mut CheckQueueHandler {}),
            Ok(SupportedRequest::CreateQueue(r)) => handle_and_send(r),
            Ok(SupportedRequest::DeleteQueue(r)) => handle_and_send(r),
            Ok(SupportedRequest::GetProperties(r)) => handle_and_send(r),
            Err(e) => {
                let x: Result<R::Response, RequestError> = Err(e);
                encode(&x)
            }
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
