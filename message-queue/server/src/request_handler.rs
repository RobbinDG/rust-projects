use crate::queue_manager::QueueManager;
use backend::request::{CheckQueue, CreateQueue, DeleteQueue, ListQueues, RequestError, RequestType};
use backend::response::ServerResponse;
use backend::status_code::Status;
use std::sync::{Arc, Mutex};

pub enum ResponseType {
    Response(ServerResponse),
    PromoteSender(ServerResponse, String),
    PromoteReceiver(ServerResponse, String),
}

pub trait RequestHandler: RequestType {
    /// TODO does this need to take a reference or can it consume the request? This avoids
    ///  cloning
    fn handle_request(
        self,
        queue_manager: Arc<Mutex<QueueManager>>,
    ) -> Result<Self::Response, RequestError>;
}

impl RequestHandler for ListQueues {
    fn handle_request(
        self,
        queue_manager: Arc<Mutex<QueueManager>>,
    ) -> Result<Self::Response, RequestError> {
        let queues_data = queue_manager
            .lock()
            .map_err(|err| RequestError::Internal("poison".to_string()))?
            .queues();
        println!("{:?}", queues_data);
        Ok(queues_data)
    }
}

impl RequestHandler for CheckQueue {
    fn handle_request(
        self,
        queue_manager: Arc<Mutex<QueueManager>>,
    ) -> Result<Self::Response, RequestError> {
        let sanitised_name = self.queue_name.replace("\n", "");
        if queue_manager
            .lock()
            .map_err(|err| RequestError::Internal("poison".to_string()))?
            .queue_exists(&sanitised_name)
        {
            Ok(Status::Exists)
        } else {
            Ok(Status::Failed)
        }
    }
}

impl RequestHandler for CreateQueue {
    fn handle_request(
        self,
        queue_manager: Arc<Mutex<QueueManager>>,
    ) -> Result<Self::Response, RequestError> {
        let mut qm = queue_manager
            .lock()
            .map_err(|err| RequestError::Internal("poison".to_string()))?;

        let sanitised_name = self.queue_name.replace("\n", "");
        if qm.queue_exists(&sanitised_name) {
            Ok(Status::Exists)
        } else {
            qm.create(sanitised_name);
            Ok(Status::Created)
        }
    }
}

impl RequestHandler for DeleteQueue {
    fn handle_request(self, queue_manager: Arc<Mutex<QueueManager>>) -> Result<Self::Response, RequestError> {
        let mut qm = queue_manager
            .lock()
            .map_err(|err| RequestError::Internal("poison".to_string()))?;

        let sanitised_name = self.queue_name.replace("\n", "");
        if qm.delete(&sanitised_name).is_some() {
            Ok(Status::Removed)
        } else {
            Ok(Status::NotFound)
        }
    }
}
