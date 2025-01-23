use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BufferProperties {
    pub system_buffer: bool,
}
