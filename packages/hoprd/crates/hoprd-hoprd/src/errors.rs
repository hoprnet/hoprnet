use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum HoprdError {
    #[error("file error: '{0}'")]
    FileError(String),

    #[error("configuration error: '{0}'")]
    ConfigError(String),

    #[error("serialization failed: '{0}'")]
    SerializationError(String),

    #[error("validation failed: '{0}'")]
    ValidationError(String),

    #[error("db failed: '{0}'")]
    DbError(#[from] utils_db::errors::DbError),
}

pub type Result<T> = std::result::Result<T, HoprdError>;
