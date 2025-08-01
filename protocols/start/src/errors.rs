use thiserror::Error;

#[derive(Error, Debug)]
pub enum StartError {
    #[error("unknown start protocol tag")]
    UnknownTag,
    #[error("invalid start protocol version")]
    InvalidVersion,
    #[error("invalid start protocol message length")]
    InvalidLength,
    #[error("unknown start protocol message")]
    UnknownMessage,
    #[error("message parse error: {0}")]
    ParseError(String),
    #[error("cbor error: {0}")]
    CborError(#[from] serde_cbor_2::Error),
}

pub type Result<T> = std::result::Result<T, StartError>;
