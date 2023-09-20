use core_crypto::errors::CryptoError;
use multiaddr::Error as MultiaddrError;
use thiserror::Error;
use utils_db::errors::DbError;
use utils_types::errors::GeneralError;

#[derive(Error, Debug)]
pub enum CoreEthereumError {
    #[error("commitment failure: {0}")]
    CommitmentError(String),

    #[error(transparent)]
    CryptoError(#[from] CryptoError),

    #[error(transparent)]
    DbError(#[from] DbError),

    #[error(transparent)]
    OtherError(#[from] GeneralError),

    #[error("{0}")]
    InvalidArguments(String),

    #[error("{0}")]
    InvalidState(String),

    #[error("channel to the given destination already exists")]
    ChannelAlreadyExists,

    #[error("channel does not exist")]
    ChannelDoesNotExist,

    #[error("channel is already closed")]
    ChannelAlreadyClosed,

    #[error("Invalid response to acknowledgement {0}")]
    InvalidResponseToAcknowledgement(String),

    #[error(transparent)]
    MultiaddrParseError(#[from] MultiaddrError),

    #[error("Ticket is not a win")]
    NotAWinningTicket,

    #[error("Error while trying to submit ticket")]
    CouldNotSubmitTicket(String),

    #[error("acknowledged {0} is in a wrong state for the operation")]
    WrongTicketState(String),

    #[error("on-chain submission of transaction failed: {0}")]
    TransactionSubmissionFailed(String),
}

pub type Result<T> = core::result::Result<T, CoreEthereumError>;

#[cfg(feature = "wasm")]
impl From<CoreEthereumError> for wasm_bindgen::JsValue {
    fn from(value: CoreEthereumError) -> Self {
        value.to_string().into()
    }
}
