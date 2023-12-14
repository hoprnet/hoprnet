pub mod api;
pub mod errors;

use std::fmt::Debug;

use core_protocol::{
    ack::config::AckProtocolConfig,
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
                    cfg.set_request_timeout(hb_cfg.timeout());
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
                    cfg.set_request_timeout(msg_cfg.timeout());
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
                    cfg.set_request_timeout(ack_cfg.timeout());
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
                    cfg.set_request_timeout(ticket_aggregation_cfg.timeout());
                    cfg
                },
            ),
            keep_alive: libp2p_swarm::keep_alive::Behaviour::default(),
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

/// Build wasm variant of `Transport` for the Node environment
#[cfg(all(feature = "wasm", not(test)))]
pub fn build_basic_transport() -> libp2p_wasm_ext::ExtTransport {
    libp2p_wasm_ext::ExtTransport::new(libp2p_wasm_ext::ffi::tcp_transport())
}

/// Build wasm variant of `Swarm`
#[cfg(all(feature = "wasm", not(test)))]
pub fn build_swarm<T: NetworkBehaviour>(
    transport: libp2p::core::transport::Boxed<(PeerId, libp2p::core::muxing::StreamMuxerBox)>,
    behavior: T,
    me: PeerId,
) -> libp2p_swarm::Swarm<T> {
    SwarmBuilder::with_wasm_executor(transport, behavior, me).build()
}

/// Build native `Transport`
#[cfg(any(not(feature = "wasm"), test))]
fn build_basic_transport() -> libp2p::tcp::Transport<libp2p::tcp::async_io::Tcp> {
    libp2p::tcp::async_io::Transport::new(libp2p::tcp::Config::default().nodelay(true))
}

/// Build native `Swarm`
#[cfg(any(not(feature = "wasm"), test))]
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
pub fn build_p2p_network(
    me: libp2p_identity::Keypair,
    ack_proto_cfg: AckProtocolConfig,
    heartbeat_proto_cfg: HeartbeatProtocolConfig,
    msg_proto_cfg: MsgProtocolConfig,
    ticket_aggregation_proto_cfg: TicketAggregationProtocolConfig,
) -> libp2p_swarm::Swarm<HoprNetworkBehavior> {
    let mut mplex_config = libp2p_mplex::MplexConfig::new();

    // libp2p default is 128
    // we use more to accomodate many concurrent messages
    // FIXME: make value configurable
    mplex_config.set_max_num_streams(512);

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

    let transport = build_basic_transport()
        .upgrade(upgrade::Version::V1)
        .authenticate(libp2p_noise::Config::new(&me).expect("signing libp2p-noise static keypair"))
        .multiplex(mplex_config)
        .timeout(std::time::Duration::from_secs(60))
        .boxed();

    let behavior = HoprNetworkBehavior::new(
        msg_proto_cfg,
        ack_proto_cfg,
        heartbeat_proto_cfg,
        ticket_aggregation_proto_cfg,
    );

    build_swarm(transport, behavior, PeerId::from(me.public()))
}

pub type HoprSwarm = libp2p_swarm::Swarm<HoprNetworkBehavior>;

#[cfg(feature = "wasm")]
pub mod wasm {
    use utils_log::logger::wasm::JsLogger;
    // use utils_misc::utils::wasm::JsResult;
    use wasm_bindgen::prelude::*;

    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator.
    #[cfg(feature = "wee_alloc")]
    #[global_allocator]
    static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

    static LOGGER: JsLogger = JsLogger {};

    #[allow(dead_code)]
    #[wasm_bindgen]
    pub fn core_p2p_initialize_crate() {
        let _ = JsLogger::install(&LOGGER, None);

        // When the `console_error_panic_hook` feature is enabled, we can call the
        // `set_panic_hook` function at least once during initialization, and then
        // we will get better error messages if our code ever panics.
        //
        // For more details see
        // https://github.com/rustwasm/console_error_panic_hook#readme
        #[cfg(feature = "console_error_panic_hook")]
        console_error_panic_hook::set_once();
    }

    // #[wasm_bindgen]
    // pub fn core_p2p_gather_metrics() -> JsResult<String> {
    //     utils_metrics::metrics::wasm::gather_all_metrics()
    // }
}
