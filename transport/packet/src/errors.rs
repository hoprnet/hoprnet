use thiserror::Error;

#[derive(Error, Debug)]
pub enum PacketError {
    #[error("error while decoding message: {0}")]
    DecodingError(String),
}

pub type Result<T> = core::result::Result<T, PacketError>;
