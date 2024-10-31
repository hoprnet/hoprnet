//! [`Session`] object providing the session functionality over the HOPR transport
//!
//! The session proxies the user interactions with the transport in order to hide the
//! advanced interactions and functionality.

extern crate core;

pub mod errors;
pub mod initiation;
mod manager;
pub mod traits;
pub mod types;

pub use manager::{SessionManager, SessionManagerConfig};

use libp2p_identity::PeerId;
#[cfg(feature = "serde")]
use {
    serde::{Deserialize, Serialize},
    serde_with::{As, DisplayFromStr},
};

pub use hopr_network_types::types::*;
pub use types::{IncomingSession, Session, SessionId, SESSION_USABLE_MTU_SIZE};

/// Capabilities of a session.
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, strum::EnumIter, strum::Display, strum::EnumString)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Capability {
    /// Frame segmentation
    Segmentation,
    /// Frame reassembly
    Retransmission,
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

    /// Target of the session.
    pub target: SealedHost,

    /// Capabilities offered by the session.
    pub capabilities: Vec<Capability>,
}
