use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApplicationLayerError {
    #[error("error while decoding message: {0}")]
    DecodingError(String),

    #[error("tag error: {0}")]
    TagError(String),

    #[error("application data payload is too large")]
    PayloadTooLarge,
}

pub type Result<T> = core::result::Result<T, ApplicationLayerError>;
