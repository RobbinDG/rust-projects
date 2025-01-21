use crate::new::queue_store::QueueStore;
use backend::protocol::new::request_error::RequestError;
use backend::protocol::request::{CheckQueue, CreateQueue, DeleteQueue, GetProperties, ListQueues};
use backend::protocol::{BufferProperties, DLXPreference, Request, Status};
use std::sync::{Arc, Mutex};

pub trait Handler<R>
where
    R: Request,
{
    fn handle(&mut self, request: &R) -> Result<R::Response, RequestError>;
}

pub struct ListQueuesHandler {
    queues: Arc<Mutex<QueueStore>>,
}

impl ListQueuesHandler {
    pub fn new(queues: Arc<Mutex<QueueStore>>) -> Self {
        Self { queues }
    }
}

impl Handler<ListQueues> for ListQueuesHandler {
    fn handle(
        &mut self,
        _: &ListQueues,
    ) -> Result<<ListQueues as Request>::Response, RequestError> {
        Ok(self.queues.lock()?.list())
    }
}

pub struct CheckQueueHandler {
    queues: Arc<Mutex<QueueStore>>,
}

impl CheckQueueHandler {
    pub fn new(queues: Arc<Mutex<QueueStore>>) -> Self {
        Self { queues }
    }
}

impl Handler<CheckQueue> for CheckQueueHandler {
    fn handle(
        &mut self,
        request: &CheckQueue,
    ) -> Result<<CheckQueue as Request>::Response, RequestError> {
        Ok(if self.queues.lock()?.exists(&request.queue_address) {
            Status::Exists
        } else {
            Status::Failed
        })
    }
}

pub struct CreateQueueHandler {
    queues: Arc<Mutex<QueueStore>>,
}

impl CreateQueueHandler {
    pub fn new(queues: Arc<Mutex<QueueStore>>) -> Self {
        Self { queues }
    }
}

impl Handler<CreateQueue> for CreateQueueHandler {
    fn handle(
        &mut self,
        request: &CreateQueue,
    ) -> Result<<CreateQueue as Request>::Response, RequestError> {
        let mut queues = self.queues.lock()?;
        Ok(if queues.exists(&request.queue_address) {
            Status::Exists
        } else {
            queues.create(request.queue_address.clone());
            Status::Created
        })
    }
}

pub struct DeleteQueueHandler {
    queues: Arc<Mutex<QueueStore>>,
}

impl DeleteQueueHandler {
    pub fn new(queues: Arc<Mutex<QueueStore>>) -> Self {
        Self { queues }
    }
}

impl Handler<DeleteQueue> for DeleteQueueHandler {
    fn handle(
        &mut self,
        request: &DeleteQueue,
    ) -> Result<<DeleteQueue as Request>::Response, RequestError> {
        let mut qm = self.queues.lock()?;

        Ok(if qm.delete(&request.queue_name) {
            Status::Removed
        } else {
            Status::NotFound
        })
    }
}

impl GetPropertiesHandler {
    pub fn new(queues: Arc<Mutex<QueueStore>>) -> Self {
        Self { queues }
    }
}

pub struct GetPropertiesHandler {
    queues: Arc<Mutex<QueueStore>>,
}

impl Handler<GetProperties> for GetPropertiesHandler {
    fn handle(
        &mut self,
        _: &GetProperties,
    ) -> Result<<GetProperties as Request>::Response, RequestError> {
        Ok(BufferProperties {
            system_buffer: false,
            dlx_preference: DLXPreference::Buffer,
        })
    }
}
