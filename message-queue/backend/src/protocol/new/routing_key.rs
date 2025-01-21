use crate::protocol::new::queue_id::QueueId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum DLXPreference {
    Default,
    Queue,
    Override(QueueId),
    Drop,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RoutingKey {
    id: QueueId,
    dlx: DLXPreference,
}
