use hopr_api::{OffchainPublicKey, graph::NetworkGraphError};
use thiserror::Error;

use crate::types::PathTelemetry;

#[derive(Error, Debug)]
pub enum ProbeError {
    #[error("probing traffic error: {0}")]
    TrafficError(NetworkGraphError<PathTelemetry>),

    #[error("error while decoding message data")]
    DecodingError,

    #[error("peer does not exist")]
    NonExistingPeer,

    #[error("error while pinging peer {0}: {1}")]
    PingerError(OffchainPublicKey, String),

    #[error("error sending probe: {0}")]
    SendError(String),

    #[error(transparent)]
    ApplicationLayerError(#[from] hopr_protocol_app::errors::ApplicationLayerError),
}

pub type Result<T> = core::result::Result<T, ProbeError>;
