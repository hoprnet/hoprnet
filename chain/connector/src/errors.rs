use thiserror::Error;
use hopr_api::chain::HoprKeyIdent;
use hopr_internal_types::prelude::ChannelId;

#[derive(Debug, Error)]
pub enum ConnectorError {
    #[error("invalid arguments: {0}")]
    InvalidArguments(&'static str),

    #[error("invalid state: {0}")]
    InvalidState(&'static str),

    #[error("account {0} does not exist")]
    AccountDoesNotExist(HoprKeyIdent),
    
    #[error("channel {0} does not exist")]
    ChannelDoesNotExist(ChannelId),

    #[error("type conversion error")]
    TypeConversion,

    #[error("backend error: {0}")]
    BackendError(anyhow::Error),

    #[error(transparent)]
    CacheError(#[from] std::sync::Arc<Self>),
    
    #[error(transparent)]
    ClientError(#[from] blokli_client::errors::BlokliClientError),

    #[error(transparent)]
    GeneralError(#[from] hopr_primitive_types::errors::GeneralError),

    #[error("error while signing transaction payload: {0}")]
    SigningError(anyhow::Error),
}