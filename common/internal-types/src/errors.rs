use hopr_crypto_types::errors::CryptoError;
use multiaddr::Error as MultiaddrError;
use thiserror::Error;

/// Enumeration of all core type related errors.
#[derive(Error, Debug)]
pub enum CoreTypesError {
    #[error("{0}")]
    InvalidInputData(String),

    #[error("failed to parse/deserialize the data of {0}")]
    ParseError(String),

    #[error("arithmetic error: {0}")]
    ArithmeticError(String),

    #[error("invalid ticket signature or wrong ticket recipient")]
    InvalidTicketRecipient,

    #[error("packet acknowledgement could not be verified")]
    InvalidAcknowledgement,

    #[error("invalid winning probability value")]
    InvalidWinningProbability,

    #[error("cannot acknowledge self-signed tickets. Ticket sender and recipient must be different")]
    LoopbackTicket,
    
    #[error("ticket is not winning")]
    TicketNotWinning,

    #[error(transparent)]
    InvalidMultiaddr(#[from] MultiaddrError),

    #[error(transparent)]
    CryptoError(#[from] CryptoError),

    #[error(transparent)]
    GeneralError(#[from] hopr_primitive_types::errors::GeneralError),
}

pub type Result<T> = core::result::Result<T, CoreTypesError>;
