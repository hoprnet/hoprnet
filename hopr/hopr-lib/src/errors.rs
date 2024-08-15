use thiserror::Error;

/// Enumeration of status errors thrown from this library.
#[derive(Error, Debug)]
pub enum HoprStatusError {
    #[error("HOPR status error: '{0}'")]
    General(String),

    #[error("HOPR status error: node not running.")]
    NotRunningError,
}

/// Enumeration of errors thrown from this library.
#[derive(Error, Debug)]
pub enum HoprLibError {
    #[error("HOPR lib Error: '{0}'")]
    GeneralError(String),

    #[error(transparent)]
    StatusError(#[from] HoprStatusError),

    #[error(transparent)]
    DatabaseBackendError(#[from] hopr_db_sql::errors::DbSqlError),

    #[error(transparent)]
    DbError(#[from] hopr_db_sql::api::errors::DbError),

    #[error(transparent)]
    TransportError(#[from] hopr_transport::errors::HoprTransportError),

    #[error(transparent)]
    ChainError(#[from] chain_actions::errors::ChainActionsError),

    #[error(transparent)]
    ChainApi(#[from] chain_api::errors::HoprChainError),

    #[error(transparent)]
    TypeError(#[from] hopr_primitive_types::errors::GeneralError),
}

pub type Result<T> = std::result::Result<T, HoprLibError>;
