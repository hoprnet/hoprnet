use thiserror::Error;

#[derive(Error, Debug)]
pub enum SessionError {
    #[error("failed to parse session message")]
    ParseError,

    #[error("the message has an incorrect tag")]
    UnknownMessageTag,
}

pub type Result<T> = std::result::Result<T, SessionError>;