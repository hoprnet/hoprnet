use thiserror::Error;

#[derive(Error, Debug)]
pub enum HoprLibError {
    #[error("HOPR lib Error: '{0}'")]
    GeneralError(String),

    #[error("'{0}'")]
    TransportError(#[from] core_transport::errors::HoprTransportError),

    #[error("'{0}'")]
    ChainError(#[from] core_ethereum_actions::errors::CoreEthereumActionsError),

    #[error("'{0}'")]
    ChainApi(#[from] core_ethereum_api::errors::HoprChainError),

    #[error("'{0}'")]
    DbError(#[from] utils_db::errors::DbError),

    #[error("'{0}'")]
    TypeError(#[from] utils_types::errors::GeneralError),
}

pub type Result<T> = std::result::Result<T, HoprLibError>;
