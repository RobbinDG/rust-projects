use crate::protocol::queue_id::QueueId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserQueueProperties {
    /// Whether this queue is a designated DLX queue. This will drop messages if they
    /// are somehow invalidated in this queue (e.g. by expiry). Setting this to `true`
    /// means that the `dlx` property is ignored.
    pub is_dlx: bool,
    /// The `dlx` preference for this queue. If a message has its DLX preference set
    /// to queue, this queue will be used (if it exists). If it does not exist or the
    /// value is set to `None`, the default DLX will be used instead.
    pub dlx: Option<QueueId>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SystemQueueProperties {
    /// Whether this queue is a system-managed one. This prevents deletion through
    /// administration request.
    pub is_system: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueueProperties {
    pub system: SystemQueueProperties,
    pub user: UserQueueProperties,
}
