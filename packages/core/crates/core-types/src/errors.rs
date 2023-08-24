use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreTypesError {
    #[error("{0}")]
    InvalidInputData(String),

    #[error("failed to parse/deserialize the data of {0}")]
    ParseError(String),

    #[error("Arithmetic error: {0}")]
    ArithmeticError(String)
}

pub type Result<T> = core::result::Result<T, CoreTypesError>;

#[cfg(feature = "wasm")]
impl From<CoreTypesError> for wasm_bindgen::JsValue {
    fn from(value: CoreTypesError) -> Self {
        value.to_string().into()
    }
}