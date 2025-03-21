use thiserror::Error;

/// All errors raised by the crate.
#[derive(Error, Debug, PartialEq)]
pub enum TransportIdentityError {
    #[error("Failed to process a multiaddress: {0}")]
    Multiaddress(String),
}

/// Result utilizing the [P2PError] as the error type.
pub type Result<T> = core::result::Result<T, TransportIdentityError>;
