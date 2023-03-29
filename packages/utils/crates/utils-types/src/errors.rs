use thiserror::Error;


/// Type for any possible error, because the traits in this modules
/// can be used in with any possible contexts (and error types).
pub type AnyError = Box<dyn std::error::Error + Send + Sync + 'static>;

/// Listing of some general re-usable errors
#[derive(Error, Debug)]
pub enum GeneralError {
    #[error("failed to parse/deserialize the data")]
    ParseError,
    #[error("input argument to the function is invalid")]
    InvalidInput,
    #[error(transparent)]
    Other(#[from] AnyError),
}

pub type Result<T> = core::result::Result<T, GeneralError>;