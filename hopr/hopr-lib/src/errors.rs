use hopr_api::node::state::HoprState;
pub use hopr_transport::errors::{HoprTransportError, ProbeError, ProtocolError};
use thiserror::Error;

/// Enumeration of status errors thrown from this library.
#[derive(Error, Debug)]
pub enum HoprStatusError {
    #[error("HOPR status general error: '{0}'")]
    General(String),

    #[error("HOPR status error: '{0}'")]
    NotThereYet(HoprState, String),
}

/// Enumeration of errors thrown from this library.
#[derive(Error, Debug)]
pub enum HoprLibError {
    #[error("HOPR lib Error: '{0}'")]
    GeneralError(String),

    #[error("configuration validation failed: {0}")]
    ConfigurationError(#[from] validator::ValidationErrors),

    #[error("database error: {0}")]
    DbError(#[source] anyhow::Error),

    #[error("chain error: {0}")]
    ChainError(#[source] anyhow::Error),

    #[error(transparent)]
    StatusError(#[from] HoprStatusError),

    #[error(transparent)]
    TransportError(#[from] HoprTransportError),

    #[error(transparent)]
    TypeError(#[from] hopr_primitive_types::errors::GeneralError),

    #[error(transparent)]
    NetworkTypeError(#[from] hopr_network_types::errors::NetworkTypeError),

    #[error("rayon thread pool queue full: {0}")]
    SpawnError(#[from] hopr_parallelize::cpu::SpawnError),

    #[error("unspecified error: {0}")]
    Other(#[source] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, HoprLibError>;

impl HoprLibError {
    pub fn chain<E: Into<anyhow::Error>>(e: E) -> Self {
        Self::ChainError(e.into())
    }

    pub fn db<E: Into<anyhow::Error>>(e: E) -> Self {
        Self::DbError(e.into())
    }

    pub fn other(e: impl Into<anyhow::Error>) -> Self {
        Self::Other(e.into())
    }
}
