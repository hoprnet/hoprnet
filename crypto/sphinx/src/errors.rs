use thiserror::Error;

#[derive(Error, Debug)]
pub enum SphinxError {
    #[error("failed to decode packet: {0}")]
    PacketDecodingError(String),

    #[error("failed to construct packet: {0}")]
    PacketConstructionError(String),

    #[error("data could not be padded")]
    PaddingError,

    #[error(transparent)]
    CryptoError(#[from] hopr_crypto_types::errors::CryptoError),

    #[error(transparent)]
    GeneralError(#[from] hopr_primitive_types::errors::GeneralError),
}
