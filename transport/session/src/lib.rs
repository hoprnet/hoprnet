//! [`Session`] object providing the session functionality over the HOPR transport
//!
//! The session proxies the user interactions with the transport in order to hide the
//! advanced interactions and functionality.

pub mod errors;
pub mod traits;
pub mod types;

use libp2p_identity::PeerId;
use serde::{Deserialize, Serialize};
pub use types::{Session, SessionId};

/// Send options for the session.
///
/// The send options specify how the path for the sent messages
/// should be generated during the session duration.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PathOptions {
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
pub struct SessionConfig {
    pub path_options: PathOptions,
    pub capabilities: Vec<Capability>,
}
