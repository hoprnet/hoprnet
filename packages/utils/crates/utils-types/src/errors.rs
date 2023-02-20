use thiserror::Error;

/// General error thrown from when operating on types.
#[derive(Error, Debug)]
pub enum GeneralError {
    #[error("failed to parse/deserialize the data")]
    ParseError,
    #[error("failed to compute the result (could be caused by invalid input)")]
    MathError
}

pub type Result<T> = core::result::Result<T, GeneralError>;