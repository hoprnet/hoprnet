use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbEntityError {
    #[error("conversion error: {0}")]
    ConversionError(String),
}

pub type Result<T> = std::result::Result<T, DbEntityError>;
