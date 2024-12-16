use crate::queue_manager::QueueManager;
use backend::protocol::request::{
    CheckQueue, CreateQueue, DeleteQueue, ListQueues, RequestError, RequestType,
};
use backend::protocol::{ServerResponse, Status};
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
        if queue_manager
            .lock()
            .map_err(|err| RequestError::Internal("poison".to_string()))?
            .queue_exists(&self.queue_address)
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

        if qm.queue_exists(&self.queue_address) {
            Ok(Status::Exists)
        } else {
            qm.create(self.queue_address);
            Ok(Status::Created)
        }
    }
}

impl RequestHandler for DeleteQueue {
    fn handle_request(
        self,
        queue_manager: Arc<Mutex<QueueManager>>,
    ) -> Result<Self::Response, RequestError> {
        let mut qm = queue_manager
            .lock()
            .map_err(|err| RequestError::Internal("poison".to_string()))?;

        if qm.delete(&self.queue_name).is_some() {
            Ok(Status::Removed)
        } else {
            Ok(Status::NotFound)
        }
    }
}
