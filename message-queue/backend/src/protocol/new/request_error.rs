use serde::{Deserialize, Serialize};
use std::sync::PoisonError;

#[derive(Debug, Serialize, Deserialize)]
pub enum RequestError {
    DecodeError,
    NotUnderstood,
    RequestHandlingError,
    PayloadEncodeError,
}

impl<T> From<PoisonError<T>> for RequestError {
    fn from(value: PoisonError<T>) -> Self {
        RequestError::RequestHandlingError
    }
}
