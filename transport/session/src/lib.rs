//! [`Session`] object providing the session functionality over the HOPR transport
//!
//! The session proxies the user interactions with the transport to hide the
//! advanced interactions and functionality.
//!
//! The [`SessionManager`] allows for automatic management of sessions via the Start protocol.
//!
//! This crate implements [RFC-0007](https://github.com/hoprnet/rfc/tree/main/rfcs/RFC-0007-session-protocol).
pub(crate) mod balancer;
pub mod errors;
mod initiation;
mod manager;
pub mod traits;
mod types;

pub use balancer::SurbBalancerConfig;
use hopr_internal_types::prelude::HoprPseudonym;
use hopr_network_types::prelude::state::{SessionFeature, SessionSocket};
pub use hopr_network_types::types::*;
pub use manager::{DispatchResult, MIN_BALANCER_SAMPLING_INTERVAL, SessionManager, SessionManagerConfig};
#[cfg(feature = "runtime-tokio")]
pub use types::transfer_session;
pub use types::{IncomingSession, ServiceId, Session, SessionId, SessionTarget, USABLE_PAYLOAD_CAPACITY_FOR_SESSION};

// TODO: resolve this in #7145
#[cfg(not(feature = "serde"))]
compile_error!("The `serde` feature currently must be enabled, see #7145");

/// Number of bytes that can be sent in a single Session protocol payload.
pub const SESSION_PAYLOAD_SIZE: usize = SessionSocket::<USABLE_PAYLOAD_CAPACITY_FOR_SESSION>::PAYLOAD_CAPACITY;

/// Capabilities of a session.
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, strum::EnumIter, strum::Display, strum::EnumString)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Capability {
    /// Frame segmentation
    Segmentation,
    /// Frame retransmission (ACK and NACK-based)
    Retransmission,
    /// Frame retransmission (only ACK-based)
    RetransmissionAckOnly,
    /// Disable packet buffering
    NoDelay,
}

impl IntoIterator for Capability {
    type IntoIter = std::vec::IntoIter<Self::Item>;
    type Item = SessionFeature;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Capability::Segmentation => vec![],
            Capability::Retransmission => vec![
                SessionFeature::AcknowledgeFrames,
                SessionFeature::RequestIncompleteFrames,
                SessionFeature::RetransmitFrames,
            ],
            Capability::RetransmissionAckOnly => {
                vec![SessionFeature::AcknowledgeFrames, SessionFeature::RetransmitFrames]
            }
            Capability::NoDelay => vec![SessionFeature::NoDelay],
        }
        .into_iter()
    }
}

/// Configuration for the session.
///
/// Relevant primarily for the client, since the server is only
/// a reactive component in regard to the session concept.
#[derive(Debug, PartialEq, Clone, smart_default::SmartDefault)]
pub struct SessionClientConfig {
    /// The forward path options for the session.
    #[default(RoutingOptions::Hops(hopr_primitive_types::bounded::BoundedSize::MIN))]
    pub forward_path_options: RoutingOptions,
    /// The return path options for the session.
    #[default(RoutingOptions::Hops(hopr_primitive_types::bounded::BoundedSize::MIN))]
    pub return_path_options: RoutingOptions,
    /// Capabilities offered by the session.
    #[default(_code = "vec![Capability::Segmentation]")]
    pub capabilities: Vec<Capability>,
    /// Optional pseudonym used for the session. Mostly useful for testing only.
    #[default(None)]
    pub pseudonym: Option<HoprPseudonym>,
    /// Enable automatic SURB management for the Session.
    #[default(Some(SurbBalancerConfig::default()))]
    pub surb_management: Option<SurbBalancerConfig>,
}
