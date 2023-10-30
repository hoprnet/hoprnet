use core_types::errors::CoreTypesError;
use thiserror::Error;
use utils_db::errors::DbError;
use utils_types::errors::GeneralError;

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("tx queue is full, retry later")]
    Retry,

    #[error("underlying transport error while sending packet: {0}")]
    TransportError(String),

    #[error("db error {0}")]
    DbError(#[from] DbError),

    #[error("Failed to notify an external process: {0}")]
    Notification(String),

    #[error("Ticket aggregation error: {0}")]
    ProtocolTicketAggregation(String),

    #[error("General error {0}")]
    GeneralError(#[from] GeneralError),

    #[error("Core error {0}")]
    CoreError(#[from] CoreTypesError),

    #[error("Failed on a logical error: {0}")]
    Logic(String),
}

pub type Result<T> = core::result::Result<T, ProtocolError>;

#[cfg(feature = "wasm")]
impl From<ProtocolError> for wasm_bindgen::JsValue {
    fn from(value: ProtocolError) -> Self {
        value.to_string().into()
    }
}
