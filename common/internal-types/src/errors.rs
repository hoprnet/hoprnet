use hopr_crypto_types::errors::CryptoError;
use multiaddr::Error as MultiaddrError;
use thiserror::Error;
use hopr_primitive_types::errors::GeneralError;

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
    GeneralError(#[from] GeneralError),
}

/// Path selection and construction errors.
#[derive(Error, Debug)]
pub enum PathError {
    #[error("path is not valid")]
    PathNotValid,

    #[error("path contains an invalid peer id: {0}")]
    InvalidPeer(String),

    #[error("path contains a unknown peer that cannot be resolved: {0}")]
    UnknownPeer(String),

    #[error("missing channel between {0} and {1}")]
    MissingChannel(String, String),

    #[error("channel between {0} and {1} is not opened")]
    ChannelNotOpened(String, String),

    #[error("path contains loop on {0}")]
    LoopsNotAllowed(String),

    #[error("cannot find {0} hop path {1} -> {2} in the channel graph")]
    PathNotFound(usize, String, String),

    #[error(transparent)]
    OtherError(#[from] GeneralError),
}

pub type Result<T> = core::result::Result<T, CoreTypesError>;
