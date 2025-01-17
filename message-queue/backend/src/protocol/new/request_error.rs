use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum RequestError {
    DecodeError,
    NotUnderstood,
    RequestHandlingError,
}