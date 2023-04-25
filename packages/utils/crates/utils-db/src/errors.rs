use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum DbError {
    #[error("failed to dump database into file: {0}")]
    DumpError(String),

    #[error("key not found")]
    NotFound,

    #[error("failed DB operation: {0}")]
    GenericError(String),
}

pub type Result<T> = std::result::Result<T, DbError>;
