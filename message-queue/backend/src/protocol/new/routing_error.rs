use std::sync::PoisonError;
use serde::{Deserialize, Serialize};

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
}

impl<T> From<PoisonError<T>> for RoutingError {
    fn from(_: PoisonError<T>) -> Self {
        RoutingError::Internal
    }
}