use thiserror::Error;

#[derive(Error, Debug)]
pub enum NetworkTypeError {
    #[error(transparent)]
    StartProtocolError(#[from] crate::start::errors::StartError),

    #[error(transparent)]
    SessionProtocolError(#[from] crate::session::errors::SessionError),

    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, NetworkTypeError>;
