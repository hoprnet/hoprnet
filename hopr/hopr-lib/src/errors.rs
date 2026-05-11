use hopr_api::node::HoprState;
pub use hopr_utils::network_types::errors::NetworkTypeError;
pub use hopr_transport::errors::{HoprTransportError, ProbeError, ProtocolError};
use thiserror::Error;

/// Enumeration of errors thrown from this library.
///
/// Consumers should prefer matching on specific variants over catching
/// [`GeneralError`](Self::GeneralError). Transport and network sub-variant types
/// ([`HoprTransportError`], [`ProbeError`], [`ProtocolError`], [`NetworkTypeError`])
/// are re-exported from this module for convenient pattern matching.
#[derive(Error, Debug)]
pub enum HoprLibError {
    /// A general-purpose error with a descriptive message.
    ///
    /// Prefer matching on specific variants when possible.
    #[error("HOPR lib error: '{0}'")]
    GeneralError(String),

    /// The [`Hopr`](crate::Hopr) object could not be built due to a missing component.
    #[error("hopr object could not be built: {0}")]
    BuilderError(&'static str),

    /// The node configuration failed validation.
    #[error("configuration validation failed: {0}")]
    ConfigurationError(#[from] validator::ValidationErrors),

    /// An error originating from the blockchain/chain connector layer.
    ///
    /// The inner error is type-erased because the chain connector is injected
    /// via trait objects. Use the `Display` output for diagnostics.
    #[error("chain error: {0}")]
    ChainError(#[source] anyhow::Error),

    /// The node is not in the required operational state.
    ///
    /// Contains the current [`HoprState`] and a message describing what was expected.
    /// REST API consumers typically map this to HTTP 412 Precondition Failed.
    #[error("node not ready ({0}): {1}")]
    NotReady(HoprState, String),

    /// An operation timed out.
    #[error("{context} timed out")]
    Timeout {
        /// Description of what timed out and how long it waited.
        context: String,
    },

    /// The node's wallet has insufficient native token balance.
    #[error("insufficient funds: {0}")]
    InsufficientFunds(String),

    /// An error from the transport layer (messaging, sessions, probing).
    ///
    /// Sub-variant types [`ProtocolError`] and [`ProbeError`] are re-exported
    /// from this module for convenient pattern matching.
    #[error(transparent)]
    TransportError(#[from] HoprTransportError),

    /// A network-layer type error (address parsing, sealed targets, IO).
    ///
    /// The [`NetworkTypeError`] type is re-exported from this module.
    #[error(transparent)]
    NetworkTypeError(#[from] NetworkTypeError),

    /// An unclassified internal error.
    ///
    /// Used for subsystems where a typed error variant does not exist.
    /// Use the `Display` output for diagnostics.
    #[error("unspecified error: {0}")]
    Other(#[source] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, HoprLibError>;

impl HoprLibError {
    pub fn chain<E: Into<anyhow::Error>>(e: E) -> Self {
        Self::ChainError(e.into())
    }

    pub fn other(e: impl Into<anyhow::Error>) -> Self {
        Self::Other(e.into())
    }
}
