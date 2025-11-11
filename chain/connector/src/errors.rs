use hopr_api::chain::HoprKeyIdent;
use hopr_internal_types::prelude::ChannelId;
use thiserror::Error;

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

    #[error("ticket is invalid or does not match the channel")]
    InvalidTicket,

    #[error("channel {0} is closed")]
    ChannelClosed(ChannelId),

    #[error("type conversion error: {0}")]
    TypeConversion(String),

    #[error("timeout while waiting for the graph to be synced")]
    ConnectionTimeout,

    #[error("backend error: {0}")]
    BackendError(anyhow::Error),

    #[error(transparent)]
    CacheError(#[from] std::sync::Arc<Self>),

    #[error(transparent)]
    ClientError(#[from] blokli_client::errors::BlokliClientError),

    #[error(transparent)]
    GeneralError(#[from] hopr_primitive_types::errors::GeneralError),

    #[error(transparent)]
    ChainTypesError(#[from] hopr_chain_types::errors::ChainTypesError),

    #[error(transparent)]
    CoreTypesError(#[from] hopr_internal_types::errors::CoreTypesError),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error("undefined error: {0}")]
    OtherError(anyhow::Error),
}
