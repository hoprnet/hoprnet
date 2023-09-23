use core_ethereum_actions::errors::CoreEthereumActionsError;
use thiserror::Error;
use utils_db::errors::DbError;

#[derive(Debug, Error)]
pub enum StrategyError {
    #[error("non-specific strategy error: {0}")]
    Other(String),

    #[error(transparent)]
    DbError(#[from] DbError),

    #[error(transparent)]
    ActionsError(#[from] CoreEthereumActionsError),
}

pub type Result<T> = std::result::Result<T, StrategyError>;

#[cfg(feature = "wasm")]
impl From<StrategyError> for wasm_bindgen::JsValue {
    fn from(value: StrategyError) -> Self {
        value.to_string().into()
    }
}
