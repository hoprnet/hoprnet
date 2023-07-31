pub mod api;
pub mod errors;

use futures::{select, StreamExt};
use libp2p::StreamProtocol;

pub use libp2p::identity;

use libp2p::identity as libp2p_identity;
use libp2p::core as libp2p_core;
use libp2p::swarm as libp2p_swarm;
use libp2p::noise as libp2p_noise;
use libp2p::request_response as libp2p_request_response;

use libp2p_identity::PeerId;
use libp2p_core::{upgrade, Transport};
use libp2p_swarm::{NetworkBehaviour, SwarmBuilder, SwarmEvent};

use serde::{Serialize, Deserialize};

use core_network::messaging::ControlMessage;
use utils_log::{info, error};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ping(ControlMessage);
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pong(ControlMessage);

pub const HOPR_HEARTBEAT_PROTOCOL_V_0_1_0: &str = "/hopr/heartbeat/0.1.0";
pub const HOPR_MESSAGE_PROTOCOL_V_0_1_0: &str = "/hopr/msg/0.1.0";
pub const HOPR_ACKNOWLEDGE_PROTOCOL_V_0_1_0: &str = "/hopr/ack/0.1.0";

// TODO: should be loaded from the HOPRD configuration
const HOPR_HEARTBEAT_CONNECTION_KEEPALIVE_SECS: u64 = 15;
const HOPR_HEARTBEAT_REQUEST_TIMEOUT_SECS: u64 = 30;

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "HoprNetworkBehaviorEvent")]
pub struct HoprNetworkBehavior {
    // TODO: consider including regular ipfs/ping/1.0.0 for socket keep alive
    heartbeat: libp2p_request_response::cbor::Behaviour<Ping, Pong>,
    keep_alive: libp2p_swarm::keep_alive::Behaviour     // run the business logic loop indefinitely
}

pub enum HoprNetworkBehaviorEvent {
    Heartbeat(libp2p_request_response::Event<Ping,Pong>),
    KeepAlive(void::Void)
}

impl From<void::Void> for HoprNetworkBehaviorEvent {
    fn from(event: void::Void) -> Self {
        Self::KeepAlive(event)
    }
}

impl From<libp2p_request_response::Event<Ping,Pong>> for HoprNetworkBehaviorEvent {
    fn from(event: libp2p_request_response::Event<Ping,Pong>) -> Self {
        Self::Heartbeat(event)
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
                    cfg.set_connection_keep_alive(std::time::Duration::from_secs(HOPR_HEARTBEAT_CONNECTION_KEEPALIVE_SECS));
                    cfg.set_request_timeout(std::time::Duration::from_secs(HOPR_HEARTBEAT_REQUEST_TIMEOUT_SECS));
                    cfg
                },
            ),
            keep_alive: libp2p_swarm::keep_alive::Behaviour::default()
        }
    }
}

pub fn build_p2p_network(me: &PeerId) -> libp2p_swarm::Swarm<HoprNetworkBehavior> {
    // TODO: this needs to be passed from above, packet key
    let id_keys = libp2p_identity::Keypair::generate_ed25519();

    let transport = libp2p_wasm_ext::ExtTransport::new(libp2p_wasm_ext::ffi::tcp_transport())
        .upgrade(upgrade::Version::V1)
        .authenticate(libp2p_noise::Config::new(&id_keys).expect("signing libp2p-noise static keypair"))
        .multiplex(libp2p_mplex::MplexConfig::default())
        .timeout(std::time::Duration::from_secs(20))
        .boxed();

    let behavior = HoprNetworkBehavior::default();

    SwarmBuilder::with_wasm_executor(transport, behavior, me.clone()).build()
}

pub async fn build_p2p_main_loop(mut swarm: libp2p_swarm::Swarm<HoprNetworkBehavior>, notifier: api::PingMechanism) {
    let a = async move {
        let mut notification = notifier.fuse();
        loop {
            select! {
                input = notification.select_next_some() => match api::Triggers::from(input) {
                    api::Triggers::Heartbeat(api::HeartbeatChallenge(peer, challenge)) => {
                        swarm.behaviour_mut().heartbeat.send_request(&peer, Ping(challenge));
                    },
                    _ => {}
                },
                event = swarm.select_next_some() => match event {
                    SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::Heartbeat(libp2p_request_response::Event::<Ping,Pong>::Message {
                        peer,
                        message:
                            libp2p_request_response::Message::<Ping,Pong>::Request {
                                request_id, request, channel
                            },
                    })) => {
                        info!("Received a heartbeat Ping request {} from {}", request_id, peer);
                        // let response = notification.register_pong;
                        // TODO: cannot use fused stream to call API functionality - need to use stream merging and unification instead.
                    },
                    SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::Heartbeat(libp2p_request_response::Event::<Ping,Pong>::Message {
                        peer,
                        message:
                            libp2p_request_response::Message::<Ping,Pong>::Response {
                                request_id, response
                            },
                    })) => {
                        todo!("Ping response comes in");
                    },
                    SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::Heartbeat(libp2p_request_response::Event::<Ping,Pong>::OutboundFailure {
                        peer, request_id, error,
                    })) => {
                        todo!("Filed to send ping request for some reason");
                    },
                    SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::Heartbeat(libp2p_request_response::Event::<Ping,Pong>::InboundFailure {..}))
                    | SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::Heartbeat(libp2p_request_response::Event::<Ping,Pong>::ResponseSent {..})) => {
                        todo!("We do not care about this at all!");
                    },
                    _ => {
                        todo!("Not relevant for protocol implementation, connection items!");
                    }
                }
            }
        }
    };
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use utils_log::logger::JsLogger;
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
