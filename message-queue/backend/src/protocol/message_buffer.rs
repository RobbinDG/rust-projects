use serde::{Deserialize, Serialize};
use crate::protocol::DLXPreference;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BufferProperties {
    pub system_buffer: bool,
    pub dlx_preference: DLXPreference,
}

pub trait MessageBuffer {
    fn properties(&self) -> BufferProperties;
    fn message_count(&self) -> usize;

}