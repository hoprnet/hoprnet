#[cfg(feature = "use-bindings")]
use hopr_bindings::exports::alloy::{contract::Error as ContractError, signers::Error as SignerError};

/// Dynamic contract result type.
pub type Result<T, E = ChainTypesError> = core::result::Result<T, E>;

/// Error when working with chain types.
#[derive(thiserror::Error, Debug)]
pub enum ChainTypesError {
    #[error("invalid state: {0}")]
    InvalidState(&'static str),

    #[error("invalid arguments: {0}")]
    InvalidArguments(&'static str),

    #[error("signing error: {0}")]
    SigningError(anyhow::Error),

    /// An error occurred while signing a hash.
    #[cfg(feature = "use-bindings")]
    #[error(transparent)]
    SignerError(#[from] SignerError),

    /// An error occurred while interacting with contracts.
    #[cfg(feature = "use-bindings")]
    #[error(transparent)]
    ContractError(#[from] ContractError),

    /// An error occurred while parsing an EIP-2718 transaction.
    #[error("error while parsing an EIP-2718 transaction: {0}")]
    ParseError(anyhow::Error),
}
