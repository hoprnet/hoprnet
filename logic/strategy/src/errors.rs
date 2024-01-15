use chain_actions::errors::CoreEthereumActionsError;
use hopr_primitive_types::errors::GeneralError;
use thiserror::Error;
use utils_db::errors::DbError;

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
