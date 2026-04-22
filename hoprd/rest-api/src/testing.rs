//! Test mocks for REST API handler testing.
//!
//! Uses `mockall::mock!` on the narrowed trait interfaces that individual
//! handlers require. Each mock covers the minimal trait surface for one
//! group of endpoints.

use std::collections::HashSet;

use hopr_lib::{
    Multiaddr, PeerId,
    api::{
        network::{Health, NetworkEvent, NetworkView},
        node::{ComponentStatus, HasNetworkView, HoprNodeOperations, HoprState},
    },
};

// ---------------------------------------------------------------------------
// Mock for HoprNodeOperations (startedz)
// ---------------------------------------------------------------------------

mockall::mock! {
    pub NodeOps {}
    impl HoprNodeOperations for NodeOps {
        fn status(&self) -> HoprState;
    }
}

// ---------------------------------------------------------------------------
// Mock for NetworkView
// ---------------------------------------------------------------------------

mockall::mock! {
    pub NetView {}
    #[allow(refining_impl_trait)]
    impl NetworkView for NetView {
        fn listening_as(&self) -> HashSet<Multiaddr>;
        fn multiaddress_of(&self, peer: &PeerId) -> Option<HashSet<Multiaddr>>;
        fn discovered_peers(&self) -> HashSet<PeerId>;
        fn connected_peers(&self) -> HashSet<PeerId>;
        fn is_connected(&self, peer: &PeerId) -> bool;
        fn health(&self) -> Health;
        fn subscribe_network_events(&self) -> futures::stream::Empty<NetworkEvent>;
    }
}

// ---------------------------------------------------------------------------
// Composite mock for HoprNodeOperations + HasNetworkView (readyz, healthyz)
// ---------------------------------------------------------------------------

/// Composite mock implementing both `HoprNodeOperations` and `HasNetworkView`.
///
/// mockall can't mock two traits with same-named methods (`status`) in one
/// `mock!` block, so we compose them manually.
pub struct ChecksNode {
    pub node_state: HoprState,
    pub net: MockNetView,
}

impl ChecksNode {
    pub fn new(state: HoprState, health: Health) -> Self {
        let mut net = MockNetView::new();
        net.expect_health().returning(move || health);
        Self {
            node_state: state,
            net,
        }
    }
}

impl HoprNodeOperations for ChecksNode {
    fn status(&self) -> HoprState {
        self.node_state
    }
}

impl HasNetworkView for ChecksNode {
    type NetworkView = MockNetView;
    fn network_view(&self) -> &MockNetView {
        &self.net
    }
    fn status(&self) -> ComponentStatus {
        ComponentStatus::Ready
    }
}

// ---------------------------------------------------------------------------
// Bare unit type for handlers that don't use hopr at all
// ---------------------------------------------------------------------------

/// For handlers bound only on `Send + Sync + 'static`
/// (`configuration`, `list_clients`, `close_client`, `authenticate`).
pub struct NoopNode;
