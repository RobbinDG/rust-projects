use crate::queue_store::QueueStore;
use crate::router::Router;
use backend::protocol::request::{
    CheckQueue, CreateQueue, DeleteQueue, GetProperties, ListQueues, Publish, Receive,
};
use backend::protocol::request_error::RequestError;
use backend::protocol::{QueueProperties, Request, Status, SystemQueueProperties};
use std::sync::{Arc, Mutex};

pub trait Handler<R>
where
    R: Request,
{
    fn handle(&mut self, request: R) -> Result<R::Response, RequestError>;
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
    fn handle(&mut self, _: ListQueues) -> Result<<ListQueues as Request>::Response, RequestError> {
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
        request: CheckQueue,
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
        request: CreateQueue,
    ) -> Result<<CreateQueue as Request>::Response, RequestError> {
        let mut queues = self.queues.lock()?;
        Ok(if queues.exists(&request.queue_address) {
            Status::Exists
        } else {
            let properties = QueueProperties {
                system: SystemQueueProperties { is_system: false },
                user: request.properties,
            };
            queues.create(request.queue_address.clone(), properties);
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
        request: DeleteQueue,
    ) -> Result<<DeleteQueue as Request>::Response, RequestError> {
        let mut qm = self.queues.lock()?;

        Ok(if qm.delete(&request.queue_name) {
            Status::Removed
        } else {
            Status::NotFound
        })
    }
}

pub struct GetPropertiesHandler {
    queues: Arc<Mutex<QueueStore>>,
}

impl GetPropertiesHandler {
    pub fn new(queues: Arc<Mutex<QueueStore>>) -> Self {
        Self { queues }
    }
}

impl Handler<GetProperties> for GetPropertiesHandler {
    fn handle(
        &mut self,
        request: GetProperties,
    ) -> Result<<GetProperties as Request>::Response, RequestError> {
        Ok(self.queues.lock()?.properties(&request.queue).map(Clone::clone))
    }
}

pub struct PublishHandler {
    router: Arc<Mutex<Router>>,
}

impl PublishHandler {
    pub fn new(router: Arc<Mutex<Router>>) -> Self {
        Self { router }
    }
}

impl Handler<Publish> for PublishHandler {
    fn handle(&mut self, request: Publish) -> Result<<Publish as Request>::Response, RequestError> {
        let mut router = self.router.lock()?;
        Ok(router.publish(request.message))
    }
}

pub struct ReceiveHandler {
    router: Arc<Mutex<Router>>,
}

impl ReceiveHandler {
    pub fn new(router: Arc<Mutex<Router>>) -> Self {
        Self { router }
    }
}

impl Handler<Receive> for ReceiveHandler {
    fn handle(&mut self, request: Receive) -> Result<<Receive as Request>::Response, RequestError> {
        let mut binding = self.router.lock()?;
        Ok(binding.receive_valid(&request.queue))
    }
}
