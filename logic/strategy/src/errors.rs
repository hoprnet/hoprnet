use chain_actions::errors::ChainActionsError;
use hopr_primitive_types::errors::GeneralError;
use thiserror::Error;
use utils_db::errors::DbError;

/// Enumerates all errors in this crate.
#[derive(Debug, Error)]
pub enum StrategyError {
    #[error("criteria to trigger the strategy were not satisfied")]
    CriteriaNotSatisfied,

    #[error("non-specific strategy error: {0}")]
    Other(String),

    #[error(transparent)]
    DbError(#[from] DbError),

    #[error(transparent)]
    ActionsError(#[from] ChainActionsError),

    #[error("lower-level error: {0}")]
    GeneralError(#[from] GeneralError),
}

pub type Result<T> = std::result::Result<T, StrategyError>;
