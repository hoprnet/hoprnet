use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum DbError {
    #[error("failed to dump database into file: {0}")]
    DumpError(String),

    #[error("key not found")]
    NotFound,

    #[error("serialization error: {0}")]
    SerializationError(String),

    #[error("deserialization error: {0}")]
    DeserializationError(String),

    #[error("invalid input values: {0}")]
    InvalidInput(String),

    #[error("failed DB operation: {0}")]
    GenericError(String),
}

pub type Result<T> = std::result::Result<T, DbError>;

#[cfg(feature = "wasm")]
impl From<DbError> for wasm_bindgen::JsValue {
    fn from(value: DbError) -> Self {
        value.to_string().into()
    }
}

#[cfg(feature = "sqlite")]
impl From<sqlx::Error> for DbError {
    fn from(value: sqlx::Error) -> Self {
        crate::errors::DbError::GenericError(value.to_string())
    }
}
