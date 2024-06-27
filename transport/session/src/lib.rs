//! [`Session`] object providing the session functionality over the HOPR transport
//!
//! The session proxies the user interactions with the transport in order to hide the
//! advanced interactions and functionality.

pub mod errors;
pub mod traits;
pub mod types;

use libp2p_identity::PeerId;
pub use types::{Session, SessionId};

/// Send options for the session.
///
/// The send options specify how the path for the sent messages
/// should be generated during the session duration.
#[derive(Debug, Clone, PartialEq)]
pub enum PathOptions {
    IntermediatePath(Vec<PeerId>),
    Hops(u16),
}
