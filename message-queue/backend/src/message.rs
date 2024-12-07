use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    payload: String, // TODO byte string?
}

impl Message {
    pub fn new(payload: String) -> Self {
        Message { payload }
    }
}