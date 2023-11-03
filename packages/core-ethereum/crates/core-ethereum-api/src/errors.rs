use thiserror::Error;

#[derive(Error, Debug)]
pub enum HoprChainError {
    #[error("API error: {0}")]
    Api(String),

    #[error("Db error: {0}")]
    Db(#[from] utils_db::errors::DbError),
}

pub type Result<T> = core::result::Result<T, HoprChainError>;
