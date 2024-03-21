use chain_rpc::errors::RpcError;
use hopr_primitive_types::errors::GeneralError;
use thiserror::Error;

/// Enumerates all Chain Actions related errors.
#[derive(Debug, Error)]
pub enum ChainActionsError {
    #[error("channel is already opened")]
    ChannelAlreadyExists,

    #[error("channel does not exist")]
    ChannelDoesNotExist,

    #[error("channel is already closed")]
    ChannelAlreadyClosed,

    #[error("channel closure time has not elapsed yet, remaining {0}s")]
    ClosureTimeHasNotElapsed(u64),

    #[error("multiaddress has been already announced on-chain")]
    AlreadyAnnounced,

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

    #[error("timeout waiting for action confirmation")]
    Timeout,

    #[error("indexer expectation has been unregistered")]
    ExpectationUnregistered,

    #[error("no channel domain_separator tag found")]
    MissingDomainSeparator,

    #[error(transparent)]
    DbError(#[from] hopr_db_api::errors::DbError),

    #[error(transparent)]
    RpcError(#[from] RpcError),

    #[error(transparent)]
    GeneralError(#[from] GeneralError),
}

pub type Result<T> = std::result::Result<T, ChainActionsError>;
