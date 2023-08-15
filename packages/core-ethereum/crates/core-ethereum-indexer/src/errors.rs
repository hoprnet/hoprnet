use ethers::core::abi::Error as AbiError;
use thiserror::Error;
use utils_db::errors::DbError;
use utils_types::{errors::GeneralError, primitives::Address};

#[derive(Error, Debug)]
pub enum CoreEthereumIndexerError {
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
}

pub type Result<T> = core::result::Result<T, CoreEthereumIndexerError>;

#[cfg(feature = "wasm")]
impl From<CoreEthereumIndexerError> for wasm_bindgen::JsValue {
    fn from(value: CoreEthereumIndexerError) -> Self {
        value.to_string().into()
    }
}
