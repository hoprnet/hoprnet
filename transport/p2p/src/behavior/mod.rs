use std::fmt::Debug;

use futures::Stream;
use hopr_transport_protocol::PeerDiscovery;
use libp2p::{identity::PublicKey, swarm::NetworkBehaviour};

/// Definition of the HOPR discovery mechanism for the network.
pub(crate) mod discovery;

/// Network Behavior definition for aggregated HOPR network functionality.
///
/// Individual network behaviors from the libp2p perspectives are aggregated
/// under this type in order to create an aggregated network behavior capable
/// of generating events for all component behaviors.
#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "HoprNetworkBehaviorEvent")]
pub struct HoprNetworkBehavior {
    discovery: discovery::Behaviour,
    pub(crate) streams: libp2p_stream::Behaviour,
    identify: libp2p::identify::Behaviour,
    autonat: libp2p::autonat::Behaviour,
}

impl Debug for HoprNetworkBehavior {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HoprNetworkBehavior").finish()
    }
}

impl HoprNetworkBehavior {
    pub fn new<T>(me: PublicKey, external_discovery_events: T) -> Self
    where
        T: Stream<Item = PeerDiscovery> + Send + 'static,
    {
        let my_peer_id = libp2p::PeerId::from_public_key(&me);

        Self {
            streams: libp2p_stream::Behaviour::new(),
            discovery: discovery::Behaviour::new(my_peer_id, external_discovery_events),
            identify: libp2p::identify::Behaviour::new(libp2p::identify::Config::new(
                "/hopr/identify/1.0.0".to_string(),
                me,
            )),
            autonat: libp2p::autonat::Behaviour::new(my_peer_id, Default::default()),
        }
    }
}

/// Aggregated network behavior event inheriting the component behaviors' events.
///
/// Necessary to allow the libp2p handler to properly distribute the events for
/// processing in the business logic loop.
#[derive(Debug)]
pub enum HoprNetworkBehaviorEvent {
    Discovery(discovery::Event),
    Identify(Box<libp2p::identify::Event>),
    Autonat(libp2p::autonat::Event),
}

// Unexpected libp2p_stream event
impl From<()> for HoprNetworkBehaviorEvent {
    fn from(_: ()) -> Self {
        panic!("Unexpected event: ()")
    }
}

impl From<discovery::Event> for HoprNetworkBehaviorEvent {
    fn from(event: discovery::Event) -> Self {
        Self::Discovery(event)
    }
}

impl From<libp2p::identify::Event> for HoprNetworkBehaviorEvent {
    fn from(event: libp2p::identify::Event) -> Self {
        Self::Identify(Box::new(event))
    }
}
impl From<libp2p::autonat::Event> for HoprNetworkBehaviorEvent {
    fn from(event: libp2p::autonat::Event) -> Self {
        Self::Autonat(event)
    }
}
