use hex::FromHexError;
use hopr_crypto_types::errors::CryptoError;
use hopr_platform::error::PlatformError;
use serde_json::Error as JsonError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KeyPairError {
    #[error("file system error: {0}")]
    FileSystemError(#[from] PlatformError),

    #[error("cryptography error: {0}")]
    CryptographyError(#[from] CryptoError),

    #[error("JSON error: {0}")]
    JsonError(#[from] JsonError),

    #[error("hex parsing ")]
    HexParsingError(#[from] FromHexError),

    #[error("key derivation error {err:?}")]
    KeyDerivationError { err: String },

    #[error("crypto error: macs do not match. Key store may have been altered.")]
    MacMismatch,

    #[error("decoding error: invalid encrypted key length {actual} but expected {expected}")]
    InvalidEncryptedKeyLength { actual: usize, expected: usize },

    #[error("invalid version")]
    InvalidVersion { actual: usize, expected: usize },

    #[error("cryptographic parameter '{name:?}' must be {expected:?} bytes")]
    InvalidParameterSize { name: String, expected: usize },

    #[error("Invalid private key size {actual} but expected {expected}")]
    InvalidPrivateKeySize { actual: usize, expected: usize },

    #[error("{0}")]
    GeneralError(String),
}

pub type Result<T> = core::result::Result<T, KeyPairError>;
