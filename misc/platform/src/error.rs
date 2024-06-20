use thiserror::Error;

#[derive(Error, Debug)]
pub enum PlatformError {
    #[error("time error: {0}")]
    TimeError(String),

    #[error("general error: {0}")]
    GeneralError(String),
}

pub type Result<T> = core::result::Result<T, PlatformError>;
