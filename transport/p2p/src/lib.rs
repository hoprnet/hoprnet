//! # P2P
//!
//! The underlying technology for managing the peer-to-peer networking used by this package is the [`rust-libp2p`](https://github.com/libp2p/rust-libp2p) library ([documentation](https://docs.libp2p.io/)).
//!
//! ## Modularity
//!
//! `rust-libp2p` is highly modular allowing for reimplmenting expected behavior using custom implementations for API
//! traits.
//!
//! This way it is possible to experiment with and combine different components of the library in order to construct a
//! specific targeted use case.
//!
//! ## `rust-libp2p` connectivity
//!
//! As per the [official documentation](https://connectivity.libp2p.io/), the connectivity types in the library are divided into the `standalone` (implementation of network over host) and `browser` (implementation of network over browser).
//!
//! Nodes that are not located behind a blocking firewall or NAT are designated as **public nodes** and can utilize the
//! `TCP` or `QUIC` connectivity, with the recommendation to use QUIC if possible.
//!
//! Browser based solutions are almost always located behind a private network or a blocking firewall and to open a
//! connection towards the standalone nodes these utilize either the `WebSocket` approach (by hijacking the `TCP`
//! connection) or the (not yet fully speced up) `WebTransport` (by hijacking the `QUIC` connection).

/// Constants exported by the crate.
pub mod constants;

/// Errors generated by the crate.
pub mod errors;

/// Raw swarm definition for the HOPR network.
pub mod swarm;

/// P2P behavior definitions for the transport level interactions not related to the HOPR protocol
mod behavior;

use std::fmt::Debug;

pub use behavior::discovery::Event as DiscoveryEvent;
use futures::{AsyncRead, AsyncWrite, Stream};
use hopr_internal_types::prelude::*;
use hopr_transport_identity::PeerId;
use hopr_transport_protocol::PeerDiscovery;
use libp2p::{StreamProtocol, autonat, swarm::NetworkBehaviour};
use rand::rngs::OsRng;

pub const MSG_ACK_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);
pub const NAT_SERVER_PROBE_INTERVAL: std::time::Duration = std::time::Duration::from_secs(30);

// Control object for the streams over the HOPR protocols
#[derive(Clone)]
pub struct HoprStreamProtocolControl {
    control: libp2p_stream::Control,
    protocol: StreamProtocol,
}

impl HoprStreamProtocolControl {
    pub fn new(control: libp2p_stream::Control, protocol: &'static str) -> Self {
        Self {
            control,
            protocol: StreamProtocol::new(protocol),
        }
    }
}

impl std::fmt::Debug for HoprStreamProtocolControl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HoprStreamProtocolControl")
            .field("protocol", &self.protocol)
            .finish()
    }
}

#[async_trait::async_trait]
impl hopr_transport_protocol::stream::BidirectionalStreamControl for HoprStreamProtocolControl {
    fn accept(
        mut self,
    ) -> Result<impl Stream<Item = (PeerId, impl AsyncRead + AsyncWrite + Send)> + Send, impl std::error::Error> {
        self.control.accept(self.protocol)
    }

    async fn open(mut self, peer: PeerId) -> Result<impl AsyncRead + AsyncWrite + Send, impl std::error::Error> {
        self.control.open_stream(peer, self.protocol).await
    }
}

/// Network Behavior definition for aggregated HOPR network functionality.
///
/// Individual network behaviors from the libp2p perspectives are aggregated
/// under this type in order to create an aggregated network behavior capable
/// of generating events for all component behaviors.
#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "HoprNetworkBehaviorEvent")]
pub struct HoprNetworkBehavior {
    discovery: behavior::discovery::Behaviour,
    streams: libp2p_stream::Behaviour,
    pub autonat_client: autonat::v2::client::Behaviour,
    pub autonat_server: autonat::v2::server::Behaviour,
}

impl Debug for HoprNetworkBehavior {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HoprNetworkBehavior").finish()
    }
}

impl HoprNetworkBehavior {
    pub fn new<T>(me: PeerId, onchain_events: T) -> Self
    where
        T: Stream<Item = PeerDiscovery> + Send + 'static,
    {
        Self {
            streams: libp2p_stream::Behaviour::new(),
            discovery: behavior::discovery::Behaviour::new(me, onchain_events),
            autonat_client: autonat::v2::client::Behaviour::new(
                OsRng,
                autonat::v2::client::Config::default().with_probe_interval(NAT_SERVER_PROBE_INTERVAL), /* TODO (jean): make this configurable */
            ),
            autonat_server: autonat::v2::server::Behaviour::new(OsRng),
        }
    }
}

/// Aggregated network behavior event inheriting the component behaviors' events.
///
/// Necessary to allow the libp2p handler to properly distribute the events for
/// processing in the business logic loop.
#[derive(Debug)]
pub enum HoprNetworkBehaviorEvent {
    Discovery(behavior::discovery::Event),
    TicketAggregation(
        libp2p::request_response::Event<Vec<TransferableWinningTicket>, std::result::Result<Ticket, String>>,
    ),
    AutonatClient(autonat::v2::client::Event),
    AutonatServer(autonat::v2::server::Event),
}

// Unexpected libp2p_stream event
impl From<()> for HoprNetworkBehaviorEvent {
    fn from(_: ()) -> Self {
        panic!("Unexpected event: ()")
    }
}

impl From<behavior::discovery::Event> for HoprNetworkBehaviorEvent {
    fn from(event: behavior::discovery::Event) -> Self {
        Self::Discovery(event)
    }
}

impl From<libp2p::request_response::Event<Vec<TransferableWinningTicket>, std::result::Result<Ticket, String>>>
    for HoprNetworkBehaviorEvent
{
    fn from(
        event: libp2p::request_response::Event<Vec<TransferableWinningTicket>, std::result::Result<Ticket, String>>,
    ) -> Self {
        Self::TicketAggregation(event)
    }
}

impl From<autonat::v2::client::Event> for HoprNetworkBehaviorEvent {
    fn from(event: autonat::v2::client::Event) -> Self {
        Self::AutonatClient(event)
    }
}

impl From<autonat::v2::server::Event> for HoprNetworkBehaviorEvent {
    fn from(event: autonat::v2::server::Event) -> Self {
        Self::AutonatServer(event)
    }
}

pub use swarm::HoprSwarm;
