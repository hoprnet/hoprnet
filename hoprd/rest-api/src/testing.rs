//! Test infrastructure for REST API handler testing.
//!
//! Provides stub types implementing the minimal hopr-api trait subsets that
//! individual handlers require. Each stub satisfies the narrowed trait bounds
//! so handlers can be tested in isolation without the full `Hopr<...>` type.

use std::collections::HashSet;

use hopr_lib::{
    Multiaddr, PeerId,
    api::{
        network::{Health, NetworkEvent, NetworkView},
        node::{ComponentStatus, HasNetworkView, HoprNodeOperations, HoprState},
    },
};

// ---------------------------------------------------------------------------
// Stub for HoprNodeOperations + HasNetworkView (checks, node endpoints)
// ---------------------------------------------------------------------------

/// Minimal stub satisfying `HoprNodeOperations + HasNetworkView`.
///
/// Used by checks endpoints (`startedz`, `readyz`, `healthyz`).
pub struct StubNode {
    pub state: HoprState,
    pub net: StubNetworkView,
}

impl StubNode {
    pub fn running_and_healthy() -> Self {
        Self {
            state: HoprState::Running,
            net: StubNetworkView {
                health: Health::Green,
            },
        }
    }

    pub fn with_state(mut self, state: HoprState) -> Self {
        self.state = state;
        self
    }

    pub fn with_health(mut self, health: Health) -> Self {
        self.net.health = health;
        self
    }
}

impl HoprNodeOperations for StubNode {
    fn status(&self) -> HoprState {
        self.state
    }
}

impl HasNetworkView for StubNode {
    type NetworkView = StubNetworkView;

    fn network_view(&self) -> &StubNetworkView {
        &self.net
    }

    fn status(&self) -> ComponentStatus {
        ComponentStatus::Ready
    }
}

/// Stub `NetworkView` returning configured health.
pub struct StubNetworkView {
    pub health: Health,
}

impl NetworkView for StubNetworkView {
    fn listening_as(&self) -> HashSet<Multiaddr> {
        HashSet::new()
    }

    fn multiaddress_of(&self, _peer: &PeerId) -> Option<HashSet<Multiaddr>> {
        None
    }

    fn discovered_peers(&self) -> HashSet<PeerId> {
        HashSet::new()
    }

    fn connected_peers(&self) -> HashSet<PeerId> {
        HashSet::new()
    }

    fn is_connected(&self, _peer: &PeerId) -> bool {
        false
    }

    fn health(&self) -> Health {
        self.health
    }

    fn subscribe_network_events(&self) -> impl futures::Stream<Item = NetworkEvent> + Send + 'static {
        futures::stream::empty()
    }
}

// ---------------------------------------------------------------------------
// Bare stub satisfying only Send + Sync + 'static
// ---------------------------------------------------------------------------

/// Stub satisfying only `Send + Sync + 'static` — for handlers that don't
/// use `state.hopr` at all (e.g., `configuration`, `list_clients`, `authenticate`).
pub struct StubUnit;
