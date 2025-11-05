use alloy::primitives::utils::UnitsError;
use hopr_chain_rpc::errors::RpcError;
use hopr_internal_types::prelude::CoreTypesError;
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

    #[error("acknowledged {0} is in a wrong state for the operation")]
    WrongTicketState(String),

    #[error("given ticket has a superseded ticket index")]
    OldTicket,

    #[error("balance is too low to perform the operation")]
    BalanceTooLow,

    #[error("channel stake is too low or too high")]
    InvalidChannelStake,

    #[error("safe does not have enough allowance to fund channel")]
    NotEnoughAllowance,

    #[error("domain separator is missing")]
    MissingDomainSeparator,

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

    #[error("node db error: {0}")]
    NodeDbError(Box<dyn std::error::Error + Send + Sync>),

    #[error(transparent)]
    IndexDbError(#[from] hopr_db_sql::errors::DbSqlError),

    #[error(transparent)]
    RpcError(#[from] RpcError),

    #[error(transparent)]
    CoreTypesError(#[from] CoreTypesError),

    #[error(transparent)]
    GeneralError(#[from] GeneralError),

    #[error(transparent)]
    UnitsError(#[from] UnitsError),
}

pub type Result<T> = std::result::Result<T, ChainActionsError>;
