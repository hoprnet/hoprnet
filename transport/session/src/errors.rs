use thiserror::Error;

/// Enumeration of errors thrown from this library.
#[derive(Error, Debug)]
pub enum TransportSessionError {
    #[error("HOPR lib Error: '{0}'")]
    GeneralError(String),
}

pub type Result<T> = std::result::Result<T, TransportSessionError>;
