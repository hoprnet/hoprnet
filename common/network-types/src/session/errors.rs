use thiserror::Error;

#[derive(Error, Debug)]
pub enum SessionError {
    #[error("error while processing frame or segment: {0}")]
    ProcessingError(String),

    #[error("failed to parse session message")]
    ParseError,

    #[error("invalid protocol version")]
    WrongVersion,

    #[error("message has an incorrect length")]
    IncorrectMessageLength,

    #[error("the message has an unknown tag")]
    UnknownMessageTag,

    #[error("session is closed")]
    SessionClosed,
}

pub type Result<T> = std::result::Result<T, SessionError>;
