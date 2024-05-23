use hopr_db_sql::errors::DbError;
use hopr_primitive_types::errors::GeneralError;
use thiserror::Error;

/// Enumerates all errors in this crate.
#[derive(Error, Debug)]
pub enum PathError {
    #[error("path is not valid")]
    PathNotValid,

    #[error("path contains an invalid peer id: {0}")]
    InvalidPeer(String),

    #[error("missing channel between {0} and {1}")]
    MissingChannel(String, String),

    #[error("channel between {0} and {1} is not opened")]
    ChannelNotOpened(String, String),

    #[error("path contains loop on {0}")]
    LoopsNotAllowed(String),

    #[error("cannot find {0} hop path {0} -> {1} in the channel graph")]
    PathNotFound(usize, String, String),

    #[error(transparent)]
    DatabaseError(#[from] DbError),

    #[error(transparent)]
    OtherError(#[from] GeneralError),
}

pub type Result<T> = std::result::Result<T, PathError>;
