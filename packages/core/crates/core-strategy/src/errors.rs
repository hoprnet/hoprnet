use thiserror::Error;
use utils_db::errors::DbError;
use utils_types::errors::GeneralError;

#[derive(Debug, Error)]
pub enum StrategyError {
    #[error(transparent)]
    DbError(#[from] DbError),

    #[error("lower-level error: {0}")]
    Other(#[from] GeneralError),
}

pub type Result<T> = std::result::Result<T, StrategyError>;

#[cfg(feature = "wasm")]
impl From<StrategyError> for wasm_bindgen::JsValue {
    fn from(value: StrategyError) -> Self {
        value.to_string().into()
    }
}
