use core_crypto::errors::CryptoError;
use thiserror::Error;
use utils_types::errors::GeneralError;

#[derive(Error, Debug)]
pub enum PacketError {
    #[error("failed to decode packet: {0}")]
    PacketDecodingError(String),

    #[error("failed to construct packet")]
    PacketConstructionError,

    #[error("packet is in invalid state")]
    InvalidPacketState,

    #[error("Proof of Relay challenge could not be verified")]
    PoRVerificationError,

    #[error(transparent)]
    CryptographicError(#[from] CryptoError),

    #[error(transparent)]
    Other(#[from] GeneralError),
}

pub type Result<T> = std::result::Result<T, PacketError>;
