use crate::new::queue_store::QueueStore;

pub struct QueueManager {
}

impl QueueManager {
    pub fn new() -> Self {
        todo!()
    }

    pub fn process_queues(&mut self, queue_store: &mut QueueStore) {
        todo!("check queues for DLX reasons, instantiate new queues for topics, etc")
    }
}
