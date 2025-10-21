use thiserror::Error;
use hopr_internal_types::prelude::ChannelId;

#[derive(Debug, Error)]
pub enum ConnectorError {
    #[error("invalid arguments: {0}")]
    InvalidArguments(&'static str),

    #[error("invalid state: {0}")]
    InvalidState(&'static str),

    #[error("channel {0} does not exist")]
    ChannelDoesNotExist(ChannelId),

    #[error("error while storing/retrieving data: {0}")]
    StorageError(anyhow::Error),

    #[error("error while signing transaction payload: {0}")]
    SigningError(anyhow::Error),
}