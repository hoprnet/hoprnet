use ethers::core::abi::Error as AbiError;
use multiaddr::Error as MultiaddrError;
use thiserror::Error;
use utils_db::errors::DbError;
use utils_types::errors::GeneralError;

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

    #[error(transparent)]
    MultiaddrParseError(#[from] MultiaddrError),
}

pub type Result<T> = core::result::Result<T, CoreEthereumIndexerError>;

#[cfg(feature = "wasm")]
impl From<CoreEthereumIndexerError> for wasm_bindgen::JsValue {
    fn from(value: CoreEthereumIndexerError) -> Self {
        value.to_string().into()
    }
}
