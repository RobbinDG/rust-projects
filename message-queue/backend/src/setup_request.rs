use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum SetupRequest {
    Admin,
    Sender(String),
    Receiver(String),
}