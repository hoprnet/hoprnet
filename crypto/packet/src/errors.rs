use std::fmt::{Debug, Display, Formatter};

use hopr_crypto_types::errors::CryptoError;
use hopr_internal_types::{errors::CoreTypesError, prelude::Ticket};
use hopr_primitive_types::errors::GeneralError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PacketError {
    #[error("failed to decode packet: {0}")]
    PacketDecodingError(String),

    #[error("failed to construct packet: {0}")]
    PacketConstructionError(String),

    #[error("Proof of Relay challenge could not be verified")]
    PoRVerificationError,

    #[error("logic error during packet processing: {0}")]
    LogicError(String),

    #[error("underlying transport error while sending packet: {0}")]
    TransportError(String),

    #[error(transparent)]
    CryptographicError(#[from] CryptoError),

    #[error(transparent)]
    CoreTypesError(#[from] CoreTypesError),

    #[error(transparent)]
    SphinxError(#[from] hopr_crypto_sphinx::errors::SphinxError),

    #[error(transparent)]
    Other(#[from] GeneralError),
}

pub type Result<T> = std::result::Result<T, PacketError>;

/// Contains errors returned by [validate_unacknowledged_ticket](crate::validation::validate_unacknowledged_ticket]).
#[derive(Debug, Clone)]
pub struct TicketValidationError {
    /// Error description.
    pub reason: String,
    /// Invalid ticket that failed to validate.
    pub ticket: Box<Ticket>,
}

impl Display for TicketValidationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.reason)
    }
}

impl std::error::Error for TicketValidationError {}
