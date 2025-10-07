use hopr_primitive_types::errors::GeneralError;
use thiserror::Error;

/// Enumerates all errors in this crate.
#[derive(Debug, Error)]
pub enum StrategyError {
    #[error("criteria to trigger the strategy were not satisfied")]
    CriteriaNotSatisfied,

    #[error("non-specific strategy error: {0}")]
    Other(Box<dyn std::error::Error + Send + Sync>),

    #[error(transparent)]
    ProtocolError(#[from] hopr_transport_protocol::errors::ProtocolError),

    #[error("lower-level error: {0}")]
    GeneralError(#[from] GeneralError),
}

pub type Result<T> = std::result::Result<T, StrategyError>;
