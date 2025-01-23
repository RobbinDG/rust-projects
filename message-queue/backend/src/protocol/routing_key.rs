use crate::protocol::queue_id::QueueId;
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
    pub id: QueueId,
    pub dlx: DLXPreference,
}

impl RoutingKey {
    pub fn new(id: QueueId, dlx: DLXPreference) -> Self {
        RoutingKey { id, dlx }
    }
}
