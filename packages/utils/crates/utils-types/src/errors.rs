use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Type for any possible error, because the traits in this modules
/// can be used in with any possible contexts (and error types).
pub type AnyError = Box<dyn std::error::Error + Send + Sync + 'static>;

/// Listing of some general re-usable errors
#[derive(Error, Debug, Serialize, Deserialize, PartialEq)]
pub enum GeneralError {
    #[error("failed to parse/deserialize the data")]
    ParseError,

    #[error("input argument to the function is invalid")]
    InvalidInput,

    #[error("non-specific error: {0}")]
    NonSpecificError(String),
}

pub type Result<T> = core::result::Result<T, GeneralError>;
