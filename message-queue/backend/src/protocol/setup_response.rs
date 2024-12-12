use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum SetupResponse {
    Disconnect,
    Admin,
    Sender(String),
    Receiver(String),
}