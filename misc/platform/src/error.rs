use thiserror::Error;

#[derive(Error, Debug)]
pub enum PlatformError {
    #[cfg(feature = "js")]
    #[error("javascript error: {0}")]
    JsError(String),

    #[error("time error: {0}")]
    TimeError(String),

    #[error("general error: {0}")]
    GeneralError(String),
}

#[cfg(feature = "js")]
impl From<wasm_bindgen::JsValue> for PlatformError {
    fn from(v: wasm_bindgen::JsValue) -> Self {
        crate::error::PlatformError::JsError(v.as_string().unwrap_or("unknown".into()))
    }
}

pub type Result<T> = core::result::Result<T, PlatformError>;
