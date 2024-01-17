use thiserror::Error;

use multiaddr::Error as MultiaddrError;

use hopr_crypto_types::errors::CryptoError;

#[derive(Error, Debug)]
pub enum CoreTypesError {
    #[error("{0}")]
    InvalidInputData(String),

    #[error("failed to parse/deserialize the data of {0}")]
    ParseError(String),

    #[error("Arithmetic error: {0}")]
    ArithmeticError(String),

    #[error("Ticket seems to be destined for a different node")]
    InvalidTicketRecipient,

    #[error("Cannot acknowledge self-signed tickets. Ticket sender and recipient must be different")]
    LoopbackTicket,

    #[error("size of the packet payload has been exceeded")]
    PayloadSizeExceeded,

    #[error(transparent)]
    InvalidMultiaddr(#[from] MultiaddrError),

    #[error(transparent)]
    CryptoError(#[from] CryptoError),

    #[error(transparent)]
    GeneralError(#[from] hopr_primitive_types::errors::GeneralError),
}

pub type Result<T> = core::result::Result<T, CoreTypesError>;
