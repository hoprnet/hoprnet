use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("general DB error: {0}")]
    General(String),
}

pub type Result<T> = std::result::Result<T, DbError>;
