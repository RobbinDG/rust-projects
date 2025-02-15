use crate::queue_store::QueueStore;
use crate::router::Router;
use crate::subscription_manager::SubscriptionManager;
use backend::protocol::client_id::ClientID;
use backend::protocol::request::{
    CheckQueue, CreateQueue, DeleteQueue, GetProperties, GetSubscription, GetTopicBreakdown,
    ListQueues, Publish, Receive, Subscribe,
};
use backend::protocol::request_error::RequestError;
use backend::protocol::{QueueProperties, Request, Status, SystemQueueProperties};
use std::sync::{Arc, Mutex};

pub trait Handler<R>
where
    R: Request,
{
    fn handle(&self, request: R, _: ClientID) -> Result<R::Response, RequestError>;
}

pub struct ListQueuesHandler {
    queues: Arc<Mutex<QueueStore>>,
    subscription_manager: Arc<Mutex<SubscriptionManager>>,
}

impl ListQueuesHandler {
    pub fn new(
        queues: Arc<Mutex<QueueStore>>,
        subscription_manager: Arc<Mutex<SubscriptionManager>>,
    ) -> Self {
        Self {
            queues,
            subscription_manager,
        }
    }
}

impl Handler<ListQueues> for ListQueuesHandler {
    fn handle(
        &self,
        _: ListQueues,
        _: ClientID,
    ) -> Result<<ListQueues as Request>::Response, RequestError> {
        let store = self.queues.lock()?;
        let queues = store.list();
        let mut subscriber_counts = self.subscription_manager.lock()?.subscriber_counts();
        let mut result = Vec::with_capacity(queues.len());
        for queue in queues {
            let subs = subscriber_counts.remove(&queue).unwrap_or(0usize);
            let messages = store.message_count(&queue);
            result.push((queue, subs, messages));
        }
        Ok(result)
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
        &self,
        request: CheckQueue,
        _: ClientID,
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
        &self,
        request: CreateQueue,
        _: ClientID,
    ) -> Result<<CreateQueue as Request>::Response, RequestError> {
        let mut queues = self.queues.lock()?;

        let properties = QueueProperties {
            system: SystemQueueProperties { is_system: false },
            user: request.properties,
        };
        if queues.create(request.queue_address.clone(), properties) {
            Ok(Status::Created)
        } else {
            Ok(Status::Exists)
        }
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
        &self,
        request: DeleteQueue,
        _: ClientID,
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
        &self,
        request: GetProperties,
        _: ClientID,
    ) -> Result<<GetProperties as Request>::Response, RequestError> {
        Ok(self
            .queues
            .lock()?
            .properties(&request.queue)
            .map(Clone::clone))
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
    fn handle(
        &self,
        request: Publish,
        _: ClientID,
    ) -> Result<<Publish as Request>::Response, RequestError> {
        let mut router = self.router.lock()?;
        Ok(router.publish(request.message))
    }
}

pub struct SubscribeHandler {
    subscription_manager: Arc<Mutex<SubscriptionManager>>,
}

impl SubscribeHandler {
    pub fn new(subscription_manager: Arc<Mutex<SubscriptionManager>>) -> Self {
        Self {
            subscription_manager,
        }
    }
}

impl Handler<Subscribe> for SubscribeHandler {
    fn handle(
        &self,
        request: Subscribe,
        client_id: ClientID,
    ) -> Result<<Subscribe as Request>::Response, RequestError> {
        Ok(
            if self
                .subscription_manager
                .lock()?
                .subscribe(client_id, request.queue)
            {
                Status::Created
            } else {
                Status::NotFound
            },
        )
    }
}

pub struct ReceiveHandler {
    subscription_manager: Arc<Mutex<SubscriptionManager>>,
    router: Arc<Mutex<Router>>,
}

impl ReceiveHandler {
    pub fn new(
        subscription_manager: Arc<Mutex<SubscriptionManager>>,
        router: Arc<Mutex<Router>>,
    ) -> Self {
        Self {
            subscription_manager,
            router,
        }
    }
}

impl Handler<Receive> for ReceiveHandler {
    fn handle(
        &self,
        _: Receive,
        client: ClientID,
    ) -> Result<<Receive as Request>::Response, RequestError> {
        let mut router = self.router.lock()?;
        let subscriptions = self.subscription_manager.lock()?;
        let queue = match subscriptions.subscription(&client) {
            Some(queue) => queue,
            None => return Ok(None),
        };
        Ok(router.receive_valid(queue, client))
    }
}

pub struct GetTopicBreakdownHandler {
    queues: Arc<Mutex<QueueStore>>,
}

impl GetTopicBreakdownHandler {
    pub fn new(queues: Arc<Mutex<QueueStore>>) -> Self {
        Self { queues }
    }
}

impl Handler<GetTopicBreakdown> for GetTopicBreakdownHandler {
    fn handle(
        &self,
        request: GetTopicBreakdown,
        _: ClientID,
    ) -> Result<<GetTopicBreakdown as Request>::Response, RequestError> {
        let binding = self.queues.lock()?;
        let subtopics = binding
            .get_topic(&request.topic_name)
            .map(|t| t.get_subtopics());
        Ok(match subtopics {
            None => None,
            Some(subtopics) => Some(
                subtopics
                    .into_iter()
                    .map(|(k, v)| (k.clone(), v.into_iter().cloned().collect()))
                    .collect(),
            ),
        })
    }
}

pub struct GetSubscriptionHandler {
    subscription_manager: Arc<Mutex<SubscriptionManager>>,
}

impl GetSubscriptionHandler {
    pub fn new(subscription_manager: Arc<Mutex<SubscriptionManager>>) -> Self {
        Self {
            subscription_manager,
        }
    }
}

impl Handler<GetSubscription> for GetSubscriptionHandler {
    fn handle(
        &self,
        _: GetSubscription,
        client: ClientID,
    ) -> Result<<GetSubscription as Request>::Response, RequestError> {
        Ok(self
            .subscription_manager
            .lock()?
            .subscription(&client)
            .cloned())
    }
}
