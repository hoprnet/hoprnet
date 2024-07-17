//! [`Session`] object providing the session functionality over the HOPR transport
//!
//! The session proxies the user interactions with the transport in order to hide the
//! advanced interactions and functionality.

pub mod errors;
pub mod traits;
pub mod types;

use libp2p_identity::PeerId;
#[cfg(feature = "serde")]
use {
    serde::{Deserialize, Serialize},
    serde_with::{As, DisplayFromStr},
};

pub use types::{Session, SessionId};

/// Send options for the session.
///
/// The send options specify how the path for the sent messages
/// should be generated during the session duration.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PathOptions {
    #[cfg_attr(feature = "serde", serde(with = "As::<Vec<DisplayFromStr>>"))]
    IntermediatePath(Vec<PeerId>),
    Hops(u16),
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Capability {
    Segmentation,
    Retransmission,
}

/// Configuration for the session.
///
/// Relevant primarily for the client, since the server is only
/// a reactive component with regards to the session concept.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SessionClientConfig {
    /// The peer to which the session should be established.
    #[cfg_attr(feature = "serde", serde(with = "As::<DisplayFromStr>"))]
    pub peer: PeerId,
    /// The fixed path options for the session.
    pub path_options: PathOptions,
    /// Capabilities offered by the session.
    pub capabilities: Vec<Capability>,
}
