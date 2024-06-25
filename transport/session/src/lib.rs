//! [`Session`] object providing the session functionality over the HOPR transport
//!
//! The session proxies the user interactions with the transport in order to hide the
//! advanced interactions and functionality.

pub mod errors;
pub mod traits;

use futures::channel::mpsc::UnboundedReceiver;
use libp2p_identity::PeerId;

use hopr_internal_types::protocol::ApplicationData;

/// Send options for the session.
///
/// The send options specify how the path for the sent messages
/// should be generated during the session duration.
#[derive(Debug, Clone)]
pub enum SendOptions {
    IntermediatePath(Vec<PeerId>),
    Hops(u16),
}

/// ID tracking the session uniquely.
///
/// Simple wrapper around the maximum range of the port like session unique identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SessionId {
    tag: u16,
    peer: PeerId,
}

impl SessionId {
    pub fn new(tag: u16, peer: PeerId) -> Self {
        Self { tag, peer }
    }

    pub fn tag(&self) -> u16 {
        self.tag
    }

    pub fn peer(&self) -> &PeerId {
        &self.peer
    }
}

#[derive(Debug)]
pub struct Session {
    id: SessionId,
    options: SendOptions,
    rx: UnboundedReceiver<ApplicationData>,
}

impl Session {
    pub fn new(id: SessionId, options: SendOptions, rx: UnboundedReceiver<ApplicationData>) -> Self {
        Self { id, options, rx }
    }

    pub fn id(&self) -> &SessionId {
        &self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn session_id_can_be_constructed_from_any_type_that_converts_to_u64() {
    //     let u8_value = 42u8;

    //     let id: SessionId = u8_value.into();

    //     assert_eq!(id.0, u8_value as u16);
    // }
}
