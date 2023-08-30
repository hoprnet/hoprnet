use thiserror::Error;

#[derive(Error, Debug)]
pub enum EventError {
    #[error("event could not be delivered, because the queue is full")]
    QueueIsFull
}

pub type Result<T> = std::result::Result<T, EventError>;