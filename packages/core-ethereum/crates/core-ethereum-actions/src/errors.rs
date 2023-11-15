use core_ethereum_rpc::errors::RpcError;
use thiserror::Error;
use utils_db::errors::DbError;
use utils_types::errors::GeneralError;

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

    #[error("network registry does not allow accessing this peer")]
    PeerAccessDenied,

    #[error("acknowledged {0} is in a wrong state for the operation")]
    WrongTicketState(String),

    #[error("ticket is not a win")]
    NotAWinningTicket,

    #[error("balance is too low to perform the operation")]
    BalanceTooLow,

    #[error("safe does not have enough allowance to fund channel")]
    NotEnoughAllowance,

    #[error("on-chain submission of transaction failed: {0}")]
    TransactionSubmissionFailed(String),

    #[error("invalid argument: {0}")]
    InvalidArguments(String),

    #[error("invalid state: {0}")]
    InvalidState(String),

    #[error(transparent)]
    DbError(#[from] DbError),

    #[error(transparent)]
    RpcError(#[from] RpcError),

    #[error(transparent)]
    GeneralError(#[from] GeneralError)
}

pub type Result<T> = std::result::Result<T, CoreEthereumActionsError>;

#[cfg(feature = "wasm")]
impl From<CoreEthereumActionsError> for wasm_bindgen::JsValue {
    fn from(value: CoreEthereumActionsError) -> Self {
        value.to_string().into()
    }
}
