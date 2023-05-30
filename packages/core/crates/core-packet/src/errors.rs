use core_crypto::errors::CryptoError;
use thiserror::Error;
use utils_db::errors::DbError;
use utils_types::errors::GeneralError;

#[derive(Error, Debug)]
pub enum PacketError {
    #[error("failed to decode packet: {0}")]
    PacketDecodingError(String),

    #[error("failed to construct packet")]
    PacketConstructionError,

    #[error("packet is in invalid state")]
    InvalidPacketState,

    #[error("packet tag already present, possible replay")]
    TagReplay,

    #[error("could not find channel to {0}")]
    ChannelNotFound(String),

    #[error("path for the packet is not valid")]
    PathNotValid,

    #[error("ticket validation failed, packet dropped: {0}")]
    TicketValidation(String),

    #[error("invalid received acknowledgement: {0}")]
    AcknowledgementValidation(String),

    #[error("Proof of Relay challenge could not be verified")]
    PoRVerificationError,

    #[error("channel {0} is out of funds")]
    OutOfFunds(String),

    #[error("tx queue is full, retry later")]
    Retry,

    #[error("underlying transport error while sending packet: {0}")]
    TransportError(String),

    #[error(transparent)]
    CryptographicError(#[from] CryptoError),

    #[error(transparent)]
    PacketDbError(#[from] DbError),

    #[error(transparent)]
    Other(#[from] GeneralError),
}

pub type Result<T> = std::result::Result<T, PacketError>;
