use ethers::core::abi::Error as AbiError;
use hopr_primitive_types::{errors::GeneralError, primitives::Address};
use thiserror::Error;
use utils_db::errors::DbError;

#[derive(Error, Debug)]
pub enum CoreEthereumIndexerError {
    #[error("{0}")]
    ProcessError(String),

    #[error(transparent)]
    DbError(#[from] DbError),

    #[error(transparent)]
    AbiError(#[from] AbiError),

    #[error(transparent)]
    GeneralError(#[from] GeneralError),

    #[error("{0}")]
    ValidationError(String),

    #[error("Address announcement without a preceding key binding.")]
    AnnounceBeforeKeyBinding,

    #[error("Address announcement contains empty Multiaddr.")]
    AnnounceEmptyMultiaddr,

    #[error("Node has already announced a key binding. Reassigning keys is not supported.")]
    UnsupportedKeyRebinding,

    #[error("Address revocation before key binding.")]
    RevocationBeforeKeyBinding,

    #[error("Could not verify account entry signature. Maybe a cross-signing issue?")]
    AccountEntrySignatureVerification,

    #[error("Received an event for a channel that is closed or for which we haven't seen an OPEN even.")]
    ChannelDoesNotExist,

    #[error("Cannot deregister inexistent MFA module")]
    MFAModuleDoesNotExist,

    #[error("Unknown smart contract. Received event from {0}")]
    UnknownContract(Address),

    #[error(transparent)]
    MultiaddrParseError(#[from] multiaddr::Error),

    #[error(transparent)]
    RpcError(#[from] chain_rpc::errors::RpcError),
}

pub type Result<T> = core::result::Result<T, CoreEthereumIndexerError>;
