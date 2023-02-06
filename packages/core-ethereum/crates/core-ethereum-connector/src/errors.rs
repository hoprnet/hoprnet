use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConnectorError {
    #[error("on-chain execution error: {0}")]
    ExecutionError(String),
    #[error("unknown error")]
    UnknownError,
}

pub type Result<T> = core::result::Result<T, ConnectorError>;