use thiserror::Error;

#[derive(Error, Debug)]
pub enum NetworkTypeError {
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("the target is sealed")]
    SealedTarget,

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, NetworkTypeError>;
