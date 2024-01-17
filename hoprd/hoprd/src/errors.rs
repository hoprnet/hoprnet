use thiserror::Error;

#[derive(Error, Debug)]
pub enum HoprdError {
    #[error("file error: '{0}'")]
    FileError(String),

    #[error("configuration error: '{0}'")]
    ConfigError(String),

    #[error("serialization failed: '{0}'")]
    SerializationError(String),

    #[error("validation failed: '{0}'")]
    ValidationError(String),
}

pub type Result<T> = std::result::Result<T, HoprdError>;
