use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Status {
    Created,
    Removed,
    Exists,
    Sent,
    Configured,
    Failed,
    NotFound,
    UnknownCommand,
    Error,
}

impl From<Status> for &str {
    fn from(value: Status) -> Self {
        match value {
            Status::Created => "created",
            Status::Removed => "removed",
            Status::Sent => "sent",
            Status::Failed => "failed",
            Status::NotFound => "not_found",
            Status::Exists => "exists",
            Status::UnknownCommand => "unknown_command",
            Status::Error => "error",
            Status::Configured => "configured",
        }
    }
}