use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BufferProperties {
    pub system_buffer: bool,
}

pub trait MessageBuffer {
    fn properties(&self) -> BufferProperties;
    fn message_count(&self) -> usize;

}