//! [`Session`] object providing the session functionality over the HOPR transport
//!
//! The session proxies the user interactions with the transport in order to hide the
//! advanced interactions and functionality.

mod errors;
mod traits;

use libp2p_identity::PeerId;

/// ID tracking the session uniquely.
///
/// Simple wrapper around the maximum range of the port like session unique identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SessionId(u16);

impl<T: Into<u16>> From<T> for SessionId {
    fn from(id: T) -> Self {
        SessionId(id.into())
    }
}

impl AsRef<u16> for SessionId {
    fn as_ref(&self) -> &u16 {
        &self.0
    }
}

pub struct Session {
    id: SessionId,
    peer: PeerId,
    // path finding params
}

impl Session {
    pub fn new(id: SessionId, peer: PeerId) -> Self {
        Self { id, peer }
    }

    pub fn id(&self) -> SessionId {
        self.id
    }

    pub fn peer(&self) -> &PeerId {
        &self.peer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_id_can_be_constructed_from_any_type_that_converts_to_u64() {
        let u8_value = 42u8;

        let id: SessionId = u8_value.into();

        assert_eq!(id.0, u8_value as u16);
    }

    #[test]
    fn session_id_can_be_dereferenced() {
        let u16_value = 42u16;

        let id: SessionId = u16_value.into();

        assert_eq!(&u16_value, id.as_ref());
    }

    #[test]
    fn session_should_be_identifiable_by_id() {
        let id = SessionId::from(42u16);
        let peer = PeerId::random();

        let session = Session::new(id, peer);

        assert_eq!(session.id(), id);
    }
}
