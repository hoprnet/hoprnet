use core_ethereum_actions::errors::CoreEthereumActionsError;
use thiserror::Error;
use utils_db::errors::DbError;
use utils_types::errors::GeneralError;

#[derive(Debug, Error)]
pub enum StrategyError {
    #[error("criteria to trigger the strategy were not satisfied")]
    CriteriaNotSatisfied,

    #[error("non-specific strategy error: {0}")]
    Other(String),

    #[error(transparent)]
    DbError(#[from] DbError),

    #[error(transparent)]
    ActionsError(#[from] CoreEthereumActionsError),

    #[error("lower-level error: {0}")]
    GeneralError(#[from] GeneralError),
}

pub type Result<T> = std::result::Result<T, StrategyError>;