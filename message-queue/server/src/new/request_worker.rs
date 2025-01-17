use crate::new::dispatcher::RequestDispatcher;
use backend::protocol::new::codec::{encode, CodecError};
use backend::protocol::new::request_error::RequestError;
use backend::protocol::request::SupportedRequest;
use backend::protocol::{Request, ResponseError};
use backend::stream_io::{StreamIO, StreamIOError};
use log::error;

pub struct RequestWorker {
    stream_io: StreamIO,
    dispatcher: RequestDispatcher,
}

impl RequestWorker {
    async fn run(mut self) -> tokio::io::Result<StreamIO> {
        loop {
            let request: Result<SupportedRequest, StreamIOError> = self.stream_io.read().await;
            let response = request.map_err(|_| RequestError::DecodeError);
            match self.dispatcher.dispatch(response).await {
                Ok(data) => {
                    if let Err(err) = self.stream_io.write(&data).await {
                        error!("Error while writing to stream: {}", err);
                        break;
                    }
                }
                Err(err) => {
                    error!("Error when encoding response: {:?}.", err)
                }
            };
        }
        Ok(self.stream_io)
    }
}
