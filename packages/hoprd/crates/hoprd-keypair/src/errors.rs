use getrandom::Error as RandomError;
use real_base::error::RealError;
use serde_json::Error as JsonError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KeyPairError {
    #[error("could not retrieve random bytes: {0}")]
    EntropyError(#[from] RandomError),

    #[error("file system error: {0}")]
    FileSystemError(#[from] RealError),

    #[error("JSON error: {0}")]
    JsonError(#[from] JsonError),

    #[error("key derivation error {err:?}")]
    KeyDerivationError { err: String },

    #[error("crypto error: macs do not match. Key store may have been altered.")]
    MacMismatch,

    #[error("decoding error: invalid encrypted key length")]
    InvalidEncryptedKeyLength { actual: usize, expected: usize },

    #[error("cryptographic parameter '{name:?}' must be {expected:?} bytes")]
    InvalidParameterSize { name: String, expected: usize },
}

pub type Result<T> = core::result::Result<T, KeyPairError>;
