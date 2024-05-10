use thiserror::Error;

/// Enumeration of errors thrown from this library.
#[derive(Error, Debug)]
pub enum HoprLibError {
    #[error("HOPR lib Error: '{0}'")]
    GeneralError(String),

    #[error("HOPR lib status error: '{0}'")]
    StatusError(String),

    #[error(transparent)]
    DatabaseError(#[from] hopr_db_api::errors::DbError),

    #[error(transparent)]
    TransportError(#[from] core_transport::errors::HoprTransportError),

    #[error(transparent)]
    ChainError(#[from] chain_actions::errors::ChainActionsError),

    #[error(transparent)]
    ChainApi(#[from] chain_api::errors::HoprChainError),

    #[error(transparent)]
    TypeError(#[from] hopr_primitive_types::errors::GeneralError),
}

pub type Result<T> = std::result::Result<T, HoprLibError>;
