use backend::protocol::queue_id::QueueId;

pub fn pretty_print_queue_dlx(dlx: &Option<QueueId>) -> String {
    match dlx {
        None => "System Default".to_string(),
        Some(id) => id.to_string(),
    }
}
