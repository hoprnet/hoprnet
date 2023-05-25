use core_crypto::errors::CryptoError;
use thiserror::Error;
use utils_db::errors::DbError;

#[derive(Error, Debug)]
pub enum CoreEthereumError {
    #[error("commitment failure: {0}")]
    CommitmentError(String),

    #[error(transparent)]
    CryptoError(#[from] CryptoError),

    #[error(transparent)]
    DbError(#[from] DbError),
}

pub type Result<T> = core::result::Result<T, CoreEthereumError>;
