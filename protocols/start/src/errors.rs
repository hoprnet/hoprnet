use thiserror::Error;

/// Lists all possible errors.
#[derive(Error, Debug)]
pub enum StartProtocolError {
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
    #[error(transparent)]
    ApplicationLayerError(#[from] hopr_protocol_app::errors::ApplicationLayerError),
}

pub type Result<T> = std::result::Result<T, StartProtocolError>;
