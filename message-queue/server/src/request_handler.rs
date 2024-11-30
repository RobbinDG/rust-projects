use crate::QueueManager;
use backend::request::{RequestError, ServerRequest};
use backend::response::ServerResponse;
use backend::status_code::Status;
use std::str;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

pub enum ResponseType {
    Response(ServerResponse),
    PromoteSender(ServerResponse, String),
    PromoteReceiver(ServerResponse, String),
}

pub struct RequestHandler {
    queue_manager: Arc<Mutex<QueueManager>>,
}


impl RequestHandler {
    pub fn new(queue_manager: Arc<Mutex<QueueManager>>) -> Self {
        Self {
            queue_manager,
        }
    }

    pub fn handle_request(&mut self, request: ServerRequest) -> Result<ResponseType, RequestError> {
        self.exec(request).map_err(|err| RequestError::Internal("Poison".to_string()))
    }

    fn exec(&mut self, request: ServerRequest) -> Result<ResponseType, PoisonError<MutexGuard<QueueManager>>> {
        match request {
            ServerRequest::ListQueues => {
                let queues_str: String = self.queue_manager.lock()?.queues().iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<&str>>().join(" ,");
                println!("{:?}", queues_str);
                Ok(ResponseType::Response(ServerResponse::from_str(queues_str.as_str())))
            }
            ServerRequest::CheckQueue(name) => {
                if self.queue_manager.lock()?.queue_exists(&name) {
                    Ok(ResponseType::Response(ServerResponse::from_status(Status::Exists)))
                } else {
                    Ok(ResponseType::Response(ServerResponse::from_status(Status::Failed)))
                }
            }
            ServerRequest::CreateQueue(name) => {
                let mut qm = self.queue_manager.lock()?;
                if qm.queue_exists(&name) {
                    Ok(ResponseType::Response(ServerResponse::from_status(Status::Exists)))
                } else {
                    qm.create(name);
                    Ok(ResponseType::Response(ServerResponse::from_status(Status::Created)))
                }
            }
            ServerRequest::MakeSender(queue_name) => {
                Ok(ResponseType::PromoteSender(ServerResponse::from_status(Status::Configured), queue_name))
            }
            ServerRequest::MakeReceiver(queue_name) => {
                Ok(ResponseType::PromoteReceiver(ServerResponse::from_status(Status::Configured), queue_name))
            }
        }
    }
}