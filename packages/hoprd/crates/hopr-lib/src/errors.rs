use core_transport::errors::HoprTransportError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HoprLibError {
    #[error("HOPR lib Error: '{0}'")]
    GeneralError(String),

    #[error("'{0}'")]
    TransportError(#[from] HoprTransportError),

    #[error("'{0}'")]
    DbError(#[from] utils_db::errors::DbError),
}

pub type Result<T> = std::result::Result<T, HoprLibError>;
