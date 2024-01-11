//! # P2P
//! 
//! The underlying technology for managing the peer-to-peer networking used by this package is the [`rust-libp2p`](https://github.com/libp2p/rust-libp2p) library ([documentation](https://docs.libp2p.io/)).
//! 
//! ## Modularity
//! 
//! `rust-libp2p` is highly modular allowing for reimplmenting expected behavior using custom implementations for API traits.
//! 
//! This way it is possible to experiment with and combine different components of the library in order to construct a specific targeted use case.
//! 
//! ## `rust-libp2p` connectivity
//! 
//! As per the [official documentation](https://connectivity.libp2p.io/), the connectivity types in the library are divided into the `standalone` (implementation of network over host) and `browser` (implementation of network over browser).
//! 
//! Nodes that are not located behind a blocking firewall or NAT are designated as **public nodes** and can utilize the `TCP` or `QUIC` connectivity, with the recommendation to use QUIC if possible.
//! 
//! Browser based solutions are almost always located behind a private network or a blocking firewall and to open a connection towards the standalone nodes these utilize either the `WebSocket` approach (by hijacking the `TCP` connection) or the (not yet fully speced up) `WebTransport` (by hijacking the `QUIC` connection).
//! 

pub mod api;
pub mod errors;

use std::fmt::Debug;

use core_protocol::{
    ack::config::AckProtocolConfig,
    config::ProtocolConfig,
    constants::{
        HOPR_ACKNOWLEDGEMENT_CONNECTION_KEEPALIVE, HOPR_ACKNOWLEDGE_PROTOCOL_V_0_1_0,
        HOPR_HEARTBEAT_CONNECTION_KEEPALIVE, HOPR_HEARTBEAT_PROTOCOL_V_0_1_0, HOPR_MESSAGE_CONNECTION_KEEPALIVE,
        HOPR_MESSAGE_PROTOCOL_V_0_1_0, HOPR_TICKET_AGGREGATION_CONNECTION_KEEPALIVE,
        HOPR_TICKET_AGGREGATION_PROTOCOL_V_0_1_0,
    },
    heartbeat::config::HeartbeatProtocolConfig,
    msg::config::MsgProtocolConfig,
    ticket_aggregation::config::TicketAggregationProtocolConfig,
};
use core_types::{acknowledgement::AcknowledgedTicket, channels::Ticket};
pub use libp2p::{
    core as libp2p_core, identity as libp2p_identity, identity, noise as libp2p_noise,
    request_response as libp2p_request_response, swarm as libp2p_swarm, StreamProtocol,
};

use libp2p_core::{upgrade, Transport};
use libp2p_identity::PeerId;
use libp2p_swarm::{NetworkBehaviour, SwarmBuilder};

use serde::{Deserialize, Serialize};

use core_network::messaging::ControlMessage;
use core_types::acknowledgement::Acknowledgement;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ping(pub ControlMessage);
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pong(pub ControlMessage, pub String);

/// Network Behavior definition for aggregated HOPR network functionality.
///
/// Individual network behaviors from the libp2p perspectives are aggregated
/// under this type in order to create an aggregated network behavior capable
/// of generating events for all component behaviors.
#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "HoprNetworkBehaviorEvent")]
pub struct HoprNetworkBehavior {
    pub heartbeat: libp2p_request_response::cbor::Behaviour<Ping, Pong>,
    pub msg: libp2p_request_response::cbor::Behaviour<Box<[u8]>, ()>,
    pub ack: libp2p_request_response::cbor::Behaviour<Acknowledgement, ()>,
    pub ticket_aggregation:
        libp2p_request_response::cbor::Behaviour<Vec<AcknowledgedTicket>, std::result::Result<Ticket, String>>,
    keep_alive: libp2p_swarm::keep_alive::Behaviour, // run the business logic loop indefinitely
}

impl Debug for HoprNetworkBehavior {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HoprNetworkBehavior").finish()
    }
}

impl HoprNetworkBehavior {
    pub fn new(
        msg_cfg: MsgProtocolConfig,
        ack_cfg: AckProtocolConfig,
        hb_cfg: HeartbeatProtocolConfig,
        ticket_aggregation_cfg: TicketAggregationProtocolConfig,
    ) -> Self {
        Self {
            heartbeat: libp2p_request_response::cbor::Behaviour::<Ping, Pong>::new(
                [(
                    StreamProtocol::new(HOPR_HEARTBEAT_PROTOCOL_V_0_1_0),
                    libp2p_request_response::ProtocolSupport::Full,
                )],
                {
                    let mut cfg = libp2p_request_response::Config::default();
                    cfg.set_connection_keep_alive(HOPR_HEARTBEAT_CONNECTION_KEEPALIVE);
                    cfg.set_request_timeout(hb_cfg.timeout);
                    cfg
                },
            ),
            msg: libp2p_request_response::cbor::Behaviour::<Box<[u8]>, ()>::new(
                [(
                    StreamProtocol::new(HOPR_MESSAGE_PROTOCOL_V_0_1_0),
                    libp2p_request_response::ProtocolSupport::Full,
                )],
                {
                    let mut cfg = libp2p_request_response::Config::default();
                    cfg.set_connection_keep_alive(HOPR_MESSAGE_CONNECTION_KEEPALIVE);
                    cfg.set_request_timeout(msg_cfg.timeout);
                    cfg
                },
            ),
            ack: libp2p_request_response::cbor::Behaviour::<Acknowledgement, ()>::new(
                [(
                    StreamProtocol::new(HOPR_ACKNOWLEDGE_PROTOCOL_V_0_1_0),
                    libp2p_request_response::ProtocolSupport::Full,
                )],
                {
                    let mut cfg = libp2p_request_response::Config::default();
                    cfg.set_connection_keep_alive(HOPR_ACKNOWLEDGEMENT_CONNECTION_KEEPALIVE);
                    cfg.set_request_timeout(ack_cfg.timeout);
                    cfg
                },
            ),
            ticket_aggregation: libp2p_request_response::cbor::Behaviour::<
                Vec<AcknowledgedTicket>,
                std::result::Result<Ticket, String>,
            >::new(
                [(
                    StreamProtocol::new(HOPR_TICKET_AGGREGATION_PROTOCOL_V_0_1_0),
                    libp2p_request_response::ProtocolSupport::Full,
                )],
                {
                    let mut cfg = libp2p_request_response::Config::default();
                    cfg.set_connection_keep_alive(HOPR_TICKET_AGGREGATION_CONNECTION_KEEPALIVE);
                    cfg.set_request_timeout(ticket_aggregation_cfg.timeout);
                    cfg
                },
            ),
            keep_alive: libp2p_swarm::keep_alive::Behaviour,
        }
    }
}

impl Default for HoprNetworkBehavior {
    fn default() -> Self {
        Self::new(
            MsgProtocolConfig::default(),
            AckProtocolConfig::default(),
            HeartbeatProtocolConfig::default(),
            TicketAggregationProtocolConfig::default(),
        )
    }
}

/// Aggregated network behavior event inheriting the component behaviors' events.
///
/// Necessary to allow the libp2p handler to properly distribute the events for
/// processing in the business logic loop.
#[derive(Debug)]
pub enum HoprNetworkBehaviorEvent {
    Heartbeat(libp2p_request_response::Event<Ping, Pong>),
    Message(libp2p_request_response::Event<Box<[u8]>, ()>),
    Acknowledgement(libp2p_request_response::Event<Acknowledgement, ()>),
    TicketAggregation(libp2p_request_response::Event<Vec<AcknowledgedTicket>, std::result::Result<Ticket, String>>),
    KeepAlive(void::Void),
}

impl From<void::Void> for HoprNetworkBehaviorEvent {
    fn from(event: void::Void) -> Self {
        Self::KeepAlive(event)
    }
}

impl From<libp2p_request_response::Event<Ping, Pong>> for HoprNetworkBehaviorEvent {
    fn from(event: libp2p_request_response::Event<Ping, Pong>) -> Self {
        Self::Heartbeat(event)
    }
}

impl From<libp2p_request_response::Event<Box<[u8]>, ()>> for HoprNetworkBehaviorEvent {
    fn from(event: libp2p_request_response::Event<Box<[u8]>, ()>) -> Self {
        Self::Message(event)
    }
}

impl From<libp2p_request_response::Event<Vec<AcknowledgedTicket>, std::result::Result<Ticket, String>>>
    for HoprNetworkBehaviorEvent
{
    fn from(
        event: libp2p_request_response::Event<Vec<AcknowledgedTicket>, std::result::Result<Ticket, String>>,
    ) -> Self {
        Self::TicketAggregation(event)
    }
}

impl From<libp2p_request_response::Event<Acknowledgement, ()>> for HoprNetworkBehaviorEvent {
    fn from(event: libp2p_request_response::Event<Acknowledgement, ()>) -> Self {
        Self::Acknowledgement(event)
    }
}

/// Build native `Swarm`
fn build_swarm<T: NetworkBehaviour>(
    transport: libp2p::core::transport::Boxed<(PeerId, libp2p::core::muxing::StreamMuxerBox)>,
    behavior: T,
    me: PeerId,
) -> libp2p_swarm::Swarm<T> {
    SwarmBuilder::with_async_std_executor(transport, behavior, me).build()
}

/// Build objects comprising the p2p network.
///
/// @return A built `Swarm` object implementing the HoprNetworkBehavior functionality
pub async fn build_p2p_network(
    me: libp2p_identity::Keypair,
    protocol_cfg: ProtocolConfig,
) -> libp2p_swarm::Swarm<HoprNetworkBehavior> {
    let mut mplex_config = libp2p_mplex::MplexConfig::new();

    // libp2p default is 128
    // we use more to accomodate many concurrent messages
    // FIXME: make value configurable
    mplex_config.set_max_num_streams(1024);

    // libp2p default is 32 Bytes
    // we use the default for now
    // FIXME: benchmark and find appropriate values
    mplex_config.set_max_buffer_size(32);

    // libp2p default is 8 KBytes
    // we use the default for now, max allowed would be 1MB
    // FIXME: benchmark and find appropriate values
    mplex_config.set_split_send_size(8 * 1024);

    // libp2p default is Block
    // Alternative is ResetStream
    // FIXME: benchmark and find appropriate values
    mplex_config.set_max_buffer_behaviour(libp2p_mplex::MaxBufferBehaviour::Block);

    let tcp_transport = libp2p::tcp::async_io::Transport::new(libp2p::tcp::Config::default().nodelay(true));
    let transport = libp2p::dns::DnsConfig::system(tcp_transport)
        .await
        .expect("p2p transport with system DNS should be obtainable");

    let transport = transport
        .upgrade(upgrade::Version::V1)
        .authenticate(libp2p_noise::Config::new(&me).expect("signing libp2p-noise static keypair"))
        .multiplex(mplex_config)
        .timeout(std::time::Duration::from_secs(60))
        .boxed();

    let behavior = HoprNetworkBehavior::new(
        protocol_cfg.msg,
        protocol_cfg.ack,
        protocol_cfg.heartbeat,
        protocol_cfg.ticket_aggregation,
    );

    build_swarm(transport, behavior, PeerId::from(me.public()))
}

pub type HoprSwarm = libp2p_swarm::Swarm<HoprNetworkBehavior>;
