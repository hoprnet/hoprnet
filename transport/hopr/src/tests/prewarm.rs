//! Gating of the forward path pre-warm on the live connection state.
//!
//! The connect event and the network view can disagree: a peer can drop between the event and the
//! task that serves it. Only a peer the network still tracks as connected may have paths built.

use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

use futures::Stream;
use hopr_api::{
    Multiaddr, PeerId,
    network::{Health, NetworkEvent},
    types::crypto::prelude::{Keypair, OffchainKeypair},
};

use crate::*;

/// A network view that lists only the peers a test declares connected.
struct FixedConnections(Vec<PeerId>);

impl NetworkView for FixedConnections {
    fn listening_as(&self) -> HashSet<Multiaddr> {
        HashSet::new()
    }

    fn multiaddress_of(&self, _peer: &PeerId) -> Option<HashSet<Multiaddr>> {
        None
    }

    fn discovered_peers(&self) -> HashSet<PeerId> {
        self.0.iter().cloned().collect()
    }

    fn connected_peers(&self) -> HashSet<PeerId> {
        self.0.iter().cloned().collect()
    }

    fn is_connected(&self, peer: &PeerId) -> bool {
        self.0.contains(peer)
    }

    fn health(&self) -> Health {
        Health::Green
    }

    fn subscribe_network_events(&self) -> impl Stream<Item = NetworkEvent> + Send + 'static {
        futures::stream::empty()
    }
}

/// A peer that dropped before the task ran, and a disconnect event, must not warm anything.
#[tokio::test]
async fn prewarm_should_warm_only_a_peer_that_is_still_connected() {
    let tracked = OffchainKeypair::random();
    let dropped = OffchainKeypair::random();
    let (tracked_id, dropped_id) = (PeerId::from(*tracked.public()), PeerId::from(*dropped.public()));

    let warmed = Arc::new(Mutex::new(Vec::new()));
    let recorder = warmed.clone();

    prewarm_forward_paths_on_connect(
        futures::stream::iter(vec![
            NetworkEvent::PeerConnected(tracked_id),
            NetworkEvent::PeerConnected(dropped_id),
            NetworkEvent::PeerDisconnected(tracked_id),
        ]),
        FixedConnections(vec![tracked_id]),
        move |peer| {
            let recorder = recorder.clone();
            async move { recorder.lock().expect("lock poisoned").push(peer) }
        },
    )
    .await;

    assert_eq!(
        vec![*tracked.public()],
        *warmed.lock().expect("lock poisoned"),
        "only the peer the network still tracks as connected is warmed"
    );
}
