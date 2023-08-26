pub mod api;
pub mod errors;

use std::fmt::Debug;

use libp2p::StreamProtocol;

pub use libp2p::identity;

use libp2p::core as libp2p_core;
pub use libp2p::identity as libp2p_identity;
use libp2p::noise as libp2p_noise;
pub use libp2p::request_response as libp2p_request_response;
pub use libp2p::swarm as libp2p_swarm;

use libp2p_core::{upgrade, Transport};
use libp2p_identity::PeerId;
use libp2p_swarm::{NetworkBehaviour, SwarmBuilder};

use serde::{Deserialize, Serialize};

use core_network::messaging::ControlMessage;
use core_types::acknowledgement::Acknowledgement;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ping(pub ControlMessage);
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pong(pub ControlMessage);

pub const HOPR_HEARTBEAT_PROTOCOL_V_0_1_0: &str = "/hopr/heartbeat/0.1.0";
pub const HOPR_MESSAGE_PROTOCOL_V_0_1_0: &str = "/hopr/msg/0.1.0";
pub const HOPR_ACKNOWLEDGE_PROTOCOL_V_0_1_0: &str = "/hopr/ack/0.1.0";

const HOPR_HEARTBEAT_CONNECTION_KEEPALIVE: std::time::Duration = std::time::Duration::from_secs(3600); // 1 hour
const HOPR_HEARTBEAT_REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

const HOPR_MESSAGE_CONNECTION_KEEPALIVE: std::time::Duration = std::time::Duration::from_secs(3600); // 1 hour
const HOPR_MESSAGE_REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(15);

const HOPR_ACKNOWLEDGEMENT_CONNECTION_KEEPALIVE: std::time::Duration = std::time::Duration::from_secs(3600); // 1 hour
const HOPR_ACKNOWLEDGEMENT_REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(15);

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "HoprNetworkBehaviorEvent")]
pub struct HoprNetworkBehavior {
    pub heartbeat: libp2p_request_response::cbor::Behaviour<Ping, Pong>,
    pub msg: libp2p_request_response::cbor::Behaviour<Box<[u8]>, ()>,
    pub ack: libp2p_request_response::cbor::Behaviour<Acknowledgement, ()>,
    keep_alive: libp2p_swarm::keep_alive::Behaviour, // run the business logic loop indefinitely
}

impl Debug for HoprNetworkBehavior {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HoprNetworkBehavior").finish()
    }
}

/// Aggregated network behavior inheriting the component behaviors.
#[derive(Debug)]
pub enum HoprNetworkBehaviorEvent {
    Heartbeat(libp2p_request_response::Event<Ping, Pong>),
    Message(libp2p_request_response::Event<Box<[u8]>, ()>),
    Acknowledgement(libp2p_request_response::Event<Acknowledgement, ()>),
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

impl From<libp2p_request_response::Event<Acknowledgement, ()>> for HoprNetworkBehaviorEvent {
    fn from(event: libp2p_request_response::Event<Acknowledgement, ()>) -> Self {
        Self::Acknowledgement(event)
    }
}

impl Default for HoprNetworkBehavior {
    fn default() -> Self {
        Self {
            heartbeat: libp2p_request_response::cbor::Behaviour::<Ping, Pong>::new(
                [(
                    StreamProtocol::new(HOPR_HEARTBEAT_PROTOCOL_V_0_1_0),
                    libp2p_request_response::ProtocolSupport::Full,
                )],
                {
                    let mut cfg = libp2p_request_response::Config::default();
                    cfg.set_connection_keep_alive(HOPR_HEARTBEAT_CONNECTION_KEEPALIVE);
                    cfg.set_request_timeout(HOPR_HEARTBEAT_REQUEST_TIMEOUT);
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
                    cfg.set_request_timeout(HOPR_MESSAGE_REQUEST_TIMEOUT);
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
                    cfg.set_request_timeout(HOPR_ACKNOWLEDGEMENT_REQUEST_TIMEOUT);
                    cfg
                },
            ),
            keep_alive: libp2p_swarm::keep_alive::Behaviour::default(),
        }
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
pub fn build_p2p_network(me: libp2p_identity::Keypair) -> libp2p_swarm::Swarm<HoprNetworkBehavior> {
    let transport = build_basic_transport()
        .upgrade(upgrade::Version::V1)
        .authenticate(libp2p_noise::Config::new(&me).expect("signing libp2p-noise static keypair"))
        .multiplex(libp2p_mplex::MplexConfig::default())
        .timeout(std::time::Duration::from_secs(60))
        .boxed();

    let behavior = HoprNetworkBehavior::default();

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
