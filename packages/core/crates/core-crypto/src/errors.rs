use thiserror::Error;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("cryptographic parameter '{name:?}' must be {expected:?} bytes")]
    InvalidParameterSize {
     name: String,
     expected: usize
    },

    #[error("input to the function has invalid size")]
    InvalidInputSize
}

pub type Result<T> = core::result::Result<T, CryptoError>;