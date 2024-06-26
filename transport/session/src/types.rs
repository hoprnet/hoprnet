use futures::channel::mpsc::UnboundedReceiver;
use libp2p_identity::PeerId;

use hopr_internal_types::protocol::ApplicationData;

use crate::{traits::SendMsg, SendOptions};

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

pub struct Session {
    id: SessionId,
    options: SendOptions,
    rx: UnboundedReceiver<ApplicationData>,
    tx: Box<dyn SendMsg>,
}

impl Session {
    pub fn new(
        id: SessionId,
        options: SendOptions,
        tx: Box<dyn SendMsg>,
        rx: UnboundedReceiver<ApplicationData>,
    ) -> Self {
        Self { id, options, rx, tx }
    }

    pub fn id(&self) -> &SessionId {
        &self.id
    }
}

#[cfg(test)]
mod tests {
    use crate::traits::MockSendMsg;

    use super::*;

    #[test]
    fn session_should_identify_with_its_own_id() {
        let id = SessionId::new(1, PeerId::random());
        let (_tx, rx) = futures::channel::mpsc::unbounded();
        let mock = MockSendMsg::new();

        let session = Session::new(id, SendOptions::Hops(1), Box::new(mock), rx);

        assert_eq!(session.id(), &id);
    }
}
