use thiserror::Error;
use utils_db::errors::DbError;
use ethers::core::abi::Error as AbiError;

#[derive(Error, Debug)]
pub enum CoreEthereumIndexerError {
    #[error(transparent)]
    DbError(#[from] DbError),

    #[error(transparent)]
    AbiError(#[from] AbiError),
}

pub type Result<T> = core::result::Result<T, CoreEthereumIndexerError>;

#[cfg(feature = "wasm")]
impl From<CoreEthereumIndexerError> for wasm_bindgen::JsValue {
    fn from(value: CoreEthereumIndexerError) -> Self {
        value.to_string().into()
    }
}
