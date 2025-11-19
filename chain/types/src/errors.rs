use hopr_bindings::exports::alloy::{contract::Error as ContractError, signers::Error as SignerError};
use thiserror::Error;

/// Dynamic contract result type.
pub type Result<T, E = ChainTypesError> = core::result::Result<T, E>;

/// Error when working with chain types.
#[derive(Error, Debug)]
pub enum ChainTypesError {
    #[error("invalid state: {0}")]
    InvalidState(&'static str),

    #[error("invalid arguments: {0}")]
    InvalidArguments(&'static str),

    #[error("signing error: {0}")]
    SigningError(anyhow::Error),

    /// An error occurred while signing a hash.
    #[error(transparent)]
    SignerError(#[from] SignerError),

    /// An error occurred while interacting with contracts.
    #[error(transparent)]
    ContractError(#[from] ContractError),

    /// An error occurred while parsing an EIP-2718 transaction.
    #[error("error while parsing an EIP-2718 transaction: {0}")]
    ParseError(anyhow::Error),
}
