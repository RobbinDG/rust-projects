use crate::queue_store::QueueStore;
use backend::protocol::new::message::Message;
use backend::protocol::new::queue_id::QueueId;
use backend::stream_io::StreamIO;
use std::collections::HashMap;

enum SubscriptionState {
    NotReady,
    Ready,
    Completed(Message),
}

pub struct SubscriptionManager {
    subscriptions: HashMap<StreamIO, (QueueId, Option<SubscriptionState>)>,
}

impl SubscriptionManager {
    pub fn new() -> Self {
        todo!()
    }

    pub fn subscribe(&mut self, receiver: StreamIO, queue_id: &QueueId) {
        todo!()
    }

    pub fn distribute(&mut self, queue_manager: &mut QueueStore) {
        for (_, (queue, state)) in &mut self.subscriptions {
            let element = state.take().map(|s| match s {
                SubscriptionState::Ready => {
                    let msg = queue_manager.receiver(queue).and_then(|mut r| r.receive());
                    if let Some(message) = msg {
                        SubscriptionState::Completed(message)
                    } else {
                        s
                    }
                }
                _ => s,
            });
            if let Some(elem) = element {
                let _ = state.insert(elem);
            }
        }
    }
}
