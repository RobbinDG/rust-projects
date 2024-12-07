use crate::queue_manager::QueueManager;
use backend::request::{
    CheckQueue, CreateQueue, ListQueues, MakeReceiver, MakeSender, RequestError, RequestType,
    SetModeResponse,
};
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
        &self,
        queue_manager: Arc<Mutex<QueueManager>>,
    ) -> Result<Self::Response, RequestError>;
}

impl RequestHandler for ListQueues {
    fn handle_request(
        &self,
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
        &self,
        queue_manager: Arc<Mutex<QueueManager>>,
    ) -> Result<Self::Response, RequestError> {
        if queue_manager
            .lock()
            .map_err(|err| RequestError::Internal("poison".to_string()))?
            .queue_exists(&self.queue_name)
        {
            Ok(Status::Exists)
        } else {
            Ok(Status::Failed)
        }
    }
}

impl RequestHandler for CreateQueue {
    fn handle_request(
        &self,
        queue_manager: Arc<Mutex<QueueManager>>,
    ) -> Result<Self::Response, RequestError> {
        let mut qm = queue_manager
            .lock()
            .map_err(|err| RequestError::Internal("poison".to_string()))?;
        if qm.queue_exists(&self.queue_name) {
            Ok(Status::Exists)
        } else {
            qm.create(self.queue_name.clone());
            Ok(Status::Created)
        }
    }
}

impl RequestHandler for MakeSender {
    fn handle_request(&self, _: Arc<Mutex<QueueManager>>) -> Result<Self::Response, RequestError> {
        Ok(SetModeResponse::Sender(self.destination_queue.clone()))
    }
}

impl RequestHandler for MakeReceiver {
    fn handle_request(&self, _: Arc<Mutex<QueueManager>>) -> Result<Self::Response, RequestError> {
        Ok(SetModeResponse::Receiver(self.origin_queue.clone()))
    }
}
