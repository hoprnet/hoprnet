use thiserror::Error;

use crate::state::HoprState;

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

    #[error(transparent)]
    StatusError(#[from] HoprStatusError),
    
    #[error(transparent)]
    TransportError(#[from] hopr_transport::errors::HoprTransportError),

    #[error(transparent)]
    TypeError(#[from] hopr_primitive_types::errors::GeneralError),

    #[error(transparent)]
    NetworkTypeError(#[from] hopr_network_types::errors::NetworkTypeError),

    #[error(transparent)]
    Other(anyhow::Error),
}

pub type Result<T> = std::result::Result<T, HoprLibError>;
