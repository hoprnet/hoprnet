//! [`Session`] object providing the session functionality over the HOPR transport
//!
//! The session proxies the user interactions with the transport in order to hide the
//! advanced interactions and functionality.

pub mod errors;
pub mod initiation;
mod manager;
pub mod traits;
pub mod types;

pub use manager::{DispatchResult, SessionManager, SessionManagerConfig};

use hopr_network_types::prelude::state::SessionFeature;
pub use hopr_network_types::types::*;
use libp2p_identity::PeerId;
pub use types::{IncomingSession, Session, SessionId, SESSION_USABLE_MTU_SIZE};
#[cfg(feature = "serde")]
use {
    serde::{Deserialize, Serialize},
    serde_with::{As, DisplayFromStr},
};

/// Capabilities of a session.
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, strum::EnumIter, strum::Display, strum::EnumString)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
                SessionFeature::RequestIncompleteFrames,
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SessionClientConfig {
    /// The peer to which the session should be established.
    #[cfg_attr(feature = "serde", serde(with = "As::<DisplayFromStr>"))]
    pub peer: PeerId,

    /// The fixed path options for the session.
    pub path_options: RoutingOptions,

    /// Protocol to be used to connect to the target
    pub target_protocol: IpProtocol,

    /// Optionally encrypted target of the session.
    pub target: SealedHost,

    /// Capabilities offered by the session.
    pub capabilities: Vec<Capability>,
}
