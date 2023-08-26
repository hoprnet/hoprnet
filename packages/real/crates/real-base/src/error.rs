use thiserror::Error;

#[cfg(feature = "wasm")]
use wasm_bindgen::JsValue;

#[derive(Error, Debug)]
pub enum RealError {
    #[cfg(feature = "wasm")]
    #[error("javascript error: {0}")]
    JsError(String),

    #[error("general error: {0}")]
    GeneralError(String),
}

#[cfg(feature = "wasm")]
impl From<JsValue> for RealError {
    fn from(v: JsValue) -> Self {
        crate::error::RealError::JsError(v.as_string().unwrap_or("unknown".into()))
    }
}

pub type Result<T> = core::result::Result<T, RealError>;
