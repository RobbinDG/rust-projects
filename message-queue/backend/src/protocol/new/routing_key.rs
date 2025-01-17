use crate::protocol::new::queue_id::QueueId;

pub enum DLXPreference {
    Default,
    Queue,
    Override(QueueId),
    Drop,
}

pub struct RoutingKey {
    id: QueueId,
    dlx: DLXPreference,
}