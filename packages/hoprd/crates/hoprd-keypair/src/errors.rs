use core_crypto::errors::CryptoError;
use getrandom::Error as RandomError;
use hex::FromHexError;
use real_base::error::RealError;
use serde_json::Error as JsonError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KeyPairError {
    #[error("could not retrieve random bytes: {0}")]
    EntropyError(#[from] RandomError),

    #[error("file system error: {0}")]
    FileSystemError(#[from] RealError),

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

#[cfg(feature = "wasm")]
impl From<KeyPairError> for wasm_bindgen::JsValue {
    fn from(value: KeyPairError) -> Self {
        value.to_string().into()
    }
}
