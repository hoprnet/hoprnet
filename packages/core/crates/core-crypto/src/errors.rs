use thiserror::Error;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("cryptographic parameter '{name:?}' must be {expected:?} bytes")]
    InvalidParameterSize {
     name: String,
     expected: usize
    },
    #[error("input to the function has invalid value or size")]
    InvalidInputValue,

    #[error("secret scalar results in an invalid EC point")]
    InvalidSecretScalar,

    #[error("elliptic curve error: {0}")]
    EllipticCurveError(#[from] elliptic_curve::Error)

}

pub type Result<T> = core::result::Result<T, CryptoError>;