use hopr_crypto_packet::errors::PacketError;
pub use hopr_transport_network::errors::NetworkError;
pub use hopr_transport_probe::errors::ProbeError;
pub use hopr_transport_protocol::errors::ProtocolError;
use hopr_transport_session::errors::TransportSessionError;
use thiserror::Error;

/// Errors produced by the crate.
#[derive(Error, Debug)]
pub enum HoprTransportError {
    #[error("API error: {0}")]
    Api(String),

    #[error("General error: {0}")]
    General(#[from] hopr_primitive_types::errors::GeneralError),

    #[error("Path error: {0}")]
    Path(#[from] hopr_internal_types::errors::PathError),

    #[error("Protocol error: {0}")]
    Protocol(#[from] ProtocolError),

    #[error("Probe error: {0}")]
    Probe(#[from] ProbeError),

    #[error("Transport session error: {0}")]
    Session(#[from] TransportSessionError),

    #[error("Packet error: {0}")]
    Packet(#[from] PacketError),

    #[error("Type error: {0}")]
    Types(#[from] hopr_internal_types::errors::CoreTypesError),

    #[error("Network monitoring error: {0}")]
    NetworkError(#[from] NetworkError),

    #[error(transparent)]
    ApplicationLayerError(#[from] hopr_protocol_app::errors::ApplicationLayerError),

    #[error(transparent)]
    Chain(anyhow::Error),

    #[error(transparent)]
    Other(anyhow::Error),
}

impl HoprTransportError {
    pub fn chain<E: Into<anyhow::Error>>(e: E) -> Self {
        Self::Chain(e.into())
    }

    pub fn other<E: Into<anyhow::Error>>(e: E) -> Self {
        Self::Other(e.into())
    }
}

/// Result produced by the crate, uses the [HoprTransportError] as the error type.
pub type Result<T> = core::result::Result<T, HoprTransportError>;
