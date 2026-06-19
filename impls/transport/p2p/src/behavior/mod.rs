use std::fmt::Debug;

use futures::Stream;
use libp2p::{identity::PublicKey, swarm::NetworkBehaviour};

use crate::PeerDiscovery;

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
    ping: libp2p::ping::Behaviour,
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

        let ping_interval = std::env::var("HOPR_INTERNAL_LIBP2P_PING_INTERVAL_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .map(std::time::Duration::from_secs)
            .unwrap_or(crate::constants::HOPR_PING_INTERVAL);

        let ping_timeout = std::env::var("HOPR_INTERNAL_LIBP2P_PING_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .map(std::time::Duration::from_secs)
            .unwrap_or(crate::constants::HOPR_PING_TIMEOUT);

        Self {
            streams: libp2p_stream::Behaviour::new(),
            discovery: discovery::Behaviour::new(my_peer_id, external_discovery_events),
            identify: libp2p::identify::Behaviour::new(libp2p::identify::Config::new(
                "/hopr/identify/1.0.0".to_string(),
                me,
            )),
            autonat: libp2p::autonat::Behaviour::new(my_peer_id, Default::default()),
            ping: libp2p::ping::Behaviour::new(
                libp2p::ping::Config::new()
                    .with_interval(ping_interval)
                    .with_timeout(ping_timeout),
            ),
        }
    }
}

/// Aggregated network behavior event inheriting the component behaviors' events.
///
/// Necessary to allow the libp2p handler to properly distribute the events for
/// processing in the business logic loop.
#[derive(Debug)]
pub enum HoprNetworkBehaviorEvent {
    Discovery(()),
    Identify(Box<libp2p::identify::Event>),
    Autonat(libp2p::autonat::Event),
    Ping(libp2p::ping::Event),
}

impl From<()> for HoprNetworkBehaviorEvent {
    fn from(event: ()) -> Self {
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

impl From<libp2p::ping::Event> for HoprNetworkBehaviorEvent {
    fn from(event: libp2p::ping::Event) -> Self {
        Self::Ping(event)
    }
}
