use crate::dispatcher::RequestDispatcher;
use backend::protocol::request_error::RequestError;
use backend::protocol::request::SupportedRequest;
use backend::stream_io::{StreamIO, StreamIOError};
use log::error;

pub struct RequestWorker {
    stream_io: StreamIO,
    dispatcher: RequestDispatcher,
}

impl RequestWorker {
    pub fn new(stream: StreamIO, dispatcher: RequestDispatcher) -> Self {
        Self {
            stream_io: stream,
            dispatcher,
        }
    }

    pub async fn run(mut self) -> tokio::io::Result<StreamIO> {
        loop {
            let request: Result<SupportedRequest, StreamIOError> = self.stream_io.read().await;
            let request = request.map_err(|_| RequestError::DecodeError);
            let response = match request {
                Ok(r) => self
                    .dispatcher
                    .dispatch(r, self.stream_io.client_id()?)
                    .await,
                Err(e) => Err(e),
            };
            if let Err(e) = self.stream_io.write_encode(&response).await {
                match e {
                    StreamIOError::Stream(err) => return Err(err),
                    StreamIOError::Codec(_) => {
                        error!("Failed to encode response.")
                    }
                }
                error!("Failed to send response to client: {:?}", e);
                break;
            }
        }
        Ok(self.stream_io)
    }
}
