use crate::request::RequestType;
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

impl RequestType for Message {
    type Response = ();
}
