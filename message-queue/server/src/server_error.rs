use std::sync::PoisonError;

#[derive(Debug)]
pub enum ServerError {
    Poison,
}


impl<T> From<PoisonError<T>> for ServerError {
    fn from(_: PoisonError<T>) -> Self {
        ServerError::Poison
    }
}