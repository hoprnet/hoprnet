use hopr_lib::GeneralError;
use thiserror::Error;

/// Enumerates all errors in this crate.
#[derive(Debug, Error)]
pub enum StrategyError {
    #[error("criteria to trigger the strategy were not satisfied")]
    CriteriaNotSatisfied,

    #[error("strategy could not perform action because action of the same type is on-going")]
    InProgress,

    #[error("non-specific strategy error: {0}")]
    Other(anyhow::Error),

    #[error(transparent)]
    HoprLib(#[from] hopr_lib::errors::HoprLibError),

    #[error("lower-level error: {0}")]
    GeneralError(#[from] GeneralError),
}

impl StrategyError {
    pub fn other<E: Into<anyhow::Error>>(e: E) -> Self {
        StrategyError::Other(e.into())
    }
}

pub type Result<T> = std::result::Result<T, StrategyError>;
