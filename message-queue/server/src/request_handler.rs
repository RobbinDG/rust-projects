use backend::request::{RequestError, ServerRequest};
use backend::response::ServerResponse;
use std::str;
use backend::status_code::Status;
use crate::QueueManager;

pub struct RequestHandler {
    queue_manager: QueueManager,
}

impl RequestHandler {
    pub fn new(queue_manager: QueueManager) -> Self {
        Self {
            queue_manager,
        }
    }

    pub fn handle_request(&mut self, request: ServerRequest) -> Result<ServerResponse, RequestError> {
        match request {
            ServerRequest::ListQueues => {
                let queues_str: String = self.queue_manager.queues().iter().map(|s| s.as_str()).collect::<Vec<&str>>().join(" ,");
                println!("{:?}", queues_str);
                Ok(ServerResponse::from_str(queues_str.as_str()))
            }
            ServerRequest::CheckQueue(name) => {
                if self.queue_manager.queue_exists(&name) {
                    Ok(ServerResponse::from_status(Status::Exists))
                } else {
                    Ok(ServerResponse::from_status(Status::Failed))
                }
            }
            ServerRequest::CreateQueue(name) => {
                if self.queue_manager.queue_exists(&name) {
                    Ok(ServerResponse::from_status(Status::Exists))
                } else {
                    self.queue_manager.create(name);
                    Ok(ServerResponse::from_status(Status::Created))
                }
            }
            ServerRequest::PutMessage(queue_name, message) => {
                // TODO check queue exists
                if self.queue_manager.push(&queue_name, message) {
                    Ok(ServerResponse::from_status(Status::Sent))
                } else {
                    Ok(ServerResponse::from_status(Status::NotFound))
                }
            }
        }
    }
}