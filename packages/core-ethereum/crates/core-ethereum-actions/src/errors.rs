use core_ethereum_misc::errors::CoreEthereumError;
use thiserror::Error;
use utils_db::errors::DbError;

#[derive(Debug, Error)]
pub enum CoreEthereumActionsError {
    #[error("channel is already opened")]
    ChannelAlreadyExists,

    #[error("channel does not exist")]
    ChannelDoesNotExist,

    #[error("channel is already closed")]
    ChannelAlreadyClosed,

    #[error("channel closure time has not elapsed yet, remaining {0}s")]
    ClosureTimeHasNotElapsed(u64),

    #[error("acknowledged {0} is in a wrong state for the operation")]
    WrongTicketState(String),

    #[error("ticket is not a win")]
    NotAWinningTicket,

    #[error("safe does not have enough allowance to fund channel")]
    NotEnoughAllowance,

    #[error("on-chain submission of transaction failed: {0}")]
    TransactionSubmissionFailed(String),

    #[error(transparent)]
    DbError(#[from] DbError),

    #[error(transparent)]
    OtherError(#[from] CoreEthereumError),
}

pub type Result<T> = std::result::Result<T, CoreEthereumActionsError>;

#[cfg(feature = "wasm")]
impl From<CoreEthereumActionsError> for wasm_bindgen::JsValue {
    fn from(value: CoreEthereumActionsError) -> Self {
        value.to_string().into()
    }
}
