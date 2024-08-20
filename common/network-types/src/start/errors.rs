use thiserror::Error;

#[derive(Debug, Error)]
pub enum StartError {
    #[error("invalid protocol message length")]
    InvalidMessageLength,

    #[error("cannot decode protocol message")]
    ParseError,
}
