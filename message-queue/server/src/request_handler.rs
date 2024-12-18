use backend::protocol::request::{
    CheckQueue, CreateQueue, DeleteQueue, ListQueues, RequestError, RequestType,
};
use backend::protocol::{ResponseError, ServerResponse, Status};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use log::info;
use crate::buffer_manager::BufferManager;
use crate::buffer_interface::BufferInterface;

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
        queue_manager: Arc<Mutex<BufferManager>>,
    ) -> Result<Self::Response, ResponseError>;
}

impl RequestHandler for ListQueues {
    fn handle_request(
        self,
        queue_manager: Arc<Mutex<BufferManager>>,
    ) -> Result<Self::Response, ResponseError> {
        let queues_data = queue_manager
            .lock()?
            .buffers();
        info!("{:?}", queues_data);
        Ok(queues_data)
    }
}

impl RequestHandler for CheckQueue {
    fn handle_request(
        self,
        queue_manager: Arc<Mutex<BufferManager>>,
    ) -> Result<Self::Response, ResponseError> {
        if queue_manager
            .lock()?
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
        queue_manager: Arc<Mutex<BufferManager>>,
    ) -> Result<Self::Response, ResponseError> {
        let mut qm = queue_manager.lock()?;

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
        queue_manager: Arc<Mutex<BufferManager>>,
    ) -> Result<Self::Response, ResponseError> {
        let mut qm = queue_manager.lock()?;

        if qm.delete(&self.queue_name).is_some() {
            Ok(Status::Removed)
        } else {
            Ok(Status::NotFound)
        }
    }
}
