use core_crypto::errors::CryptoError;
use thiserror::Error;
use utils_db::errors::DbError;

#[derive(Error, Debug)]
pub enum CoreEthereumError {
    #[error("commitment failure: {0}")]
    CommitmentError(String),

    #[error(transparent)]
    CryptoError(#[from] CryptoError),

    #[error(transparent)]
    DbError(#[from] DbError),

    #[error("Invalid response to acknowledgement {0}")]
    InvalidResponseToAcknowledgement(String),

    #[error("Ticket is not a win")]
    NotAWinningTicket,

    #[error("Error while trying to submit ticket")]
    CouldNotSubmitTicket(String),
}

pub type Result<T> = core::result::Result<T, CoreEthereumError>;

#[cfg(feature = "wasm")]
impl From<CoreEthereumError> for wasm_bindgen::JsValue {
    fn from(value: CoreEthereumError) -> Self {
        value.to_string().into()
    }
}
