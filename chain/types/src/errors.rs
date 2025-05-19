use alloy::{contract::Error as ContractError, signers::Error as SignerError};
use thiserror::Error;

/// Dynamic contract result type.
pub type Result<T, E = ChainTypesError> = core::result::Result<T, E>;

/// Error when working with chain types.
#[derive(Error, Debug)]
pub enum ChainTypesError {
    /// An error occured while signing a hash.
    #[error(transparent)]
    SignerError(#[from] SignerError),

    /// An error occured while interacting with contracts.
    #[error(transparent)]
    ContractError(#[from] ContractError),
}
