//! [`Session`] object providing the session functionality over the HOPR transport
//!
//! The session proxies the user interactions with the transport in order to hide the
//! advanced interactions and functionality.
pub(crate) mod balancer;
pub mod errors;
mod initiation;
mod manager;
pub mod traits;
mod types;

pub use hopr_network_types::types::*;
pub use manager::{DispatchResult, SessionManager, SessionManagerConfig};
pub use types::{IncomingSession, ServiceId, Session, SessionId, SessionTarget, SESSION_USABLE_MTU_SIZE};

#[cfg(feature = "runtime-tokio")]
pub use types::transfer_session;

use hopr_network_types::prelude::state::SessionFeature;
use hopr_primitive_types::prelude::Address;

use crate::balancer::SurbBalancerConfig;
use hopr_internal_types::prelude::HoprPseudonym;

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
    type Item = SessionFeature;
    type IntoIter = std::vec::IntoIter<Self::Item>;

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
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", cfg_eval::cfg_eval, serde_with::serde_as)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SessionClientConfig {
    /// The peer to which the session should be established.
    #[cfg_attr(feature = "serde", serde_as(as = "serde_with::DisplayFromStr"))]
    pub peer: Address,
    /// The forward path options for the session.
    pub forward_path_options: RoutingOptions,
    /// The return path options for the session.
    pub return_path_options: RoutingOptions,
    /// Contains target protocol and optionally encrypted target of the session.
    pub target: SessionTarget,
    /// Capabilities offered by the session.
    pub capabilities: Vec<Capability>,
    /// Optional pseudonym used for the session. Mostly useful for testing only.
    pub pseudonym: Option<HoprPseudonym>,
    /// Enable automatic SURB management for the Session.
    pub surb_management: Option<SurbBalancerConfig>,
}
