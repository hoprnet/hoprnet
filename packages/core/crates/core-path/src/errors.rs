use thiserror::Error;

#[derive(Error, Debug)]
pub enum PathError {

    #[error("path is not valid")]
    PathNotValid,

    #[error("path contains an invalid peer id: {0}")]
    InvalidPeer(String),
}

pub type Result<T> = std::result::Result<T, PathError>;