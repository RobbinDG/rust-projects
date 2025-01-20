use crate::protocol::request::Request;
use crate::protocol::BufferAddress;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DLXPreference {
    // Choice(BufferAddress),
    Buffer,
    Default,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    payload: String, // TODO byte string?
    dlx: DLXPreference,
}

impl Message {
    pub fn new(payload: String) -> Self {
        Message {
            payload,
            dlx: DLXPreference::Default,
        }
    }

    pub fn dlx(&self) -> &DLXPreference {
        &self.dlx
    }
}

impl Request for Message {
    type Response = ();
}
