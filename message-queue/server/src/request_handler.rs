use crate::buffer_interface::BufferInterface;
use crate::buffer_manager::BufferManager;
use crate::server_error::ServerError;
use backend::protocol::request::{
    CheckQueue, CreateQueue, DeleteQueue, GetProperties, ListQueues, RequestType,
};
use backend::protocol::{BufferProperties, ResponseError, Status};
use std::sync::{Arc, Mutex};

pub trait RequestHandler: RequestType {
    /// TODO does this need to take a reference or can it consume the request? This avoids
    ///  cloning
    fn handle_request(
        self,
        queue_manager: Arc<Mutex<BufferManager>>,
    ) -> Result<Result<Self::Response, ResponseError>, ServerError>;
}

impl RequestHandler for ListQueues {
    fn handle_request(
        self,
        queue_manager: Arc<Mutex<BufferManager>>,
    ) -> Result<Result<Self::Response, ResponseError>, ServerError> {
        Ok(Ok(queue_manager.lock()?.buffers()))
    }
}

impl RequestHandler for CheckQueue {
    fn handle_request(
        self,
        queue_manager: Arc<Mutex<BufferManager>>,
    ) -> Result<Result<Self::Response, ResponseError>, ServerError> {
        Ok(if queue_manager.lock()?.queue_exists(&self.queue_address) {
            Ok(Status::Exists)
        } else {
            Ok(Status::Failed)
        })
    }
}

impl RequestHandler for CreateQueue {
    fn handle_request(
        self,
        queue_manager: Arc<Mutex<BufferManager>>,
    ) -> Result<Result<Self::Response, ResponseError>, ServerError> {
        let mut qm = queue_manager.lock()?;

        Ok(if qm.queue_exists(&self.queue_address) {
            Ok(Status::Exists)
        } else {
            qm.create(
                self.queue_address,
                BufferProperties {
                    system_buffer: false,
                },
            );
            Ok(Status::Created)
        })
    }
}

impl RequestHandler for DeleteQueue {
    fn handle_request(
        self,
        queue_manager: Arc<Mutex<BufferManager>>,
    ) -> Result<Result<Self::Response, ResponseError>, ServerError> {
        let mut qm = queue_manager.lock()?;

        Ok(if qm.delete(&self.queue_name).is_some() {
            Ok(Status::Removed)
        } else {
            Ok(Status::NotFound)
        })
    }
}

impl RequestHandler for GetProperties {
    fn handle_request(
        self,
        queue_manager: Arc<Mutex<BufferManager>>,
    ) -> Result<Result<Self::Response, ResponseError>, ServerError> {
        let mut qm = queue_manager.lock()?;
        Ok(qm
            .buffer_properties(&self.buffer)
            .ok_or(ResponseError::ExecFailed(Status::NotFound)))
    }
}
