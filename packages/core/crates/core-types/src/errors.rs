use thiserror::Error;

use core_crypto::errors::CryptoError;

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

    #[error(transparent)]
    CryptoError(#[from] CryptoError),

    #[error("Cannot acknowledge self-signed tickets. Ticket sender and recipient must be different")]
    LoopbackTicket,
}

pub type Result<T> = core::result::Result<T, CoreTypesError>;

#[cfg(feature = "wasm")]
impl From<CoreTypesError> for wasm_bindgen::JsValue {
    fn from(value: CoreTypesError) -> Self {
        value.to_string().into()
    }
}
