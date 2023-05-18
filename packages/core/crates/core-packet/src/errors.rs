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

    #[error("ticket validation failed, packet dropped")]
    TicketValidation,

    #[error("Proof of Relay challenge could not be verified")]
    PoRVerificationError,

    #[error("cannot create ticket - channel {0} is out of funds")]
    OutOfFunds(String),

    #[error(transparent)]
    CryptographicError(#[from] CryptoError),

    #[error(transparent)]
    PacketDbError(#[from] DbError),

    #[error(transparent)]
    Other(#[from] GeneralError),
}

pub type Result<T> = std::result::Result<T, PacketError>;
