use serde::{Deserialize, Serialize};
use std::sync::PoisonError;

#[derive(Debug, Serialize, Deserialize)]
pub enum RoutingError {
    /// The message was dropped because it was dead-lettered and the
    /// DLX preference required it to be dropped. This is usually the case when
    /// a message is removed after having previously been dead-lettered.
    DropOnDLX,
    /// The destination queue does not exist.
    NotFound,
    /// When the message was dropped because the router's internal state is poisoned.
    Internal,
    /// When there are no valid recipients for a message. This usually applies to unused topics.
    NoRecipients,
    /// When a message couldn't be published to the DLX because of the DLX
    /// queue failing to accept it.
    DLXFailed,
}

impl<T> From<PoisonError<T>> for RoutingError {
    fn from(_: PoisonError<T>) -> Self {
        RoutingError::Internal
    }
}