use hopr_primitive_types::errors::GeneralError;
use thiserror::Error;

/// Lists all errors in this crate.
#[derive(Error, Debug)]
pub enum PathError {
    #[error("path is not valid")]
    PathNotValid,

    #[error("path contains an invalid peer id: {0}")]
    InvalidPeer(String),

    #[error("path contains a unknown peer that cannot be resolved: {0}")]
    UnknownPeer(String),

    #[error("missing channel between {0} and {1}")]
    MissingChannel(String, String),

    #[error("channel between {0} and {1} is not opened")]
    ChannelNotOpened(String, String),

    #[error("path contains loop on {0}")]
    LoopsNotAllowed(String),

    #[error("cannot find {0} hop path {1} -> {2} in the channel graph")]
    PathNotFound(usize, String, String),

    #[error(transparent)]
    OtherError(#[from] GeneralError),
}

pub type Result<T> = std::result::Result<T, PathError>;
