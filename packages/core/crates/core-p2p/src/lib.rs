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
use utils_log::{debug, info, error};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ping(ControlMessage);
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pong(ControlMessage);

use futures_concurrency::stream::Merge;

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

#[derive(Debug, Clone, PartialEq)]
pub enum Inputs {
    Heartbeat(api::HeartbeatChallenge),
    ManualPing(api::ManualPingChallenge),
    MixerMessage(String),       // TODO: This should hold the `Packet` object
}

impl From<api::HeartbeatChallenge> for Inputs {
    fn from(value: api::HeartbeatChallenge) -> Self {
        Self::Heartbeat(value)
    }
}

impl From<api::ManualPingChallenge> for Inputs {
    fn from(value: api::ManualPingChallenge) -> Self {
        Self::ManualPing(value)
    }
}


pub async fn build_p2p_main_loop(swarm: libp2p_swarm::Swarm<HoprNetworkBehavior>,
    heartbeat_requests: api::HeartbeatRequester, heartbeat_responds: api::HeartbeatResponder,
    manual_ping_requests: api::ManualPingRequester, manual_ping_responds: api::HeartbeatResponder)
{
    let mut swarm = swarm;
    let mut heartbeat_responds = heartbeat_responds;
    let mut manual_ping_responds = manual_ping_responds;

    let mut active_manual_pings: std::collections::HashSet<libp2p_request_response::RequestId> = std::collections::HashSet::new();

    // TODO: return this loop
    let a = async move {
        let mut inputs = (
            heartbeat_requests.map(Inputs::Heartbeat),
            manual_ping_requests.map(Inputs::ManualPing)).merge().fuse();
        loop {
            select! {
                input = inputs.select_next_some() => match input {
                    Inputs::Heartbeat(api::HeartbeatChallenge(peer, challenge)) => {
                        swarm.behaviour_mut().heartbeat.send_request(&peer, Ping(challenge));
                    },
                    Inputs::ManualPing(api::ManualPingChallenge(peer, challenge)) => {
                        let req_id = swarm.behaviour_mut().heartbeat.send_request(&peer, Ping(challenge));
                        active_manual_pings.insert(req_id);
                    }
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
                        let challenge_response = api::HeartbeatResponder::generate_challenge_response(&request.0);
                        match swarm.behaviour_mut().heartbeat.send_response(channel, Pong(challenge_response)) {
                            Ok(_) => {},
                            Err(_) => {
                                error!("An error occured during the ping response, channel is either closed or timed out.");
                            }    
                        };
                    },
                    SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::Heartbeat(libp2p_request_response::Event::<Ping,Pong>::Message {
                        peer,
                        message:
                            libp2p_request_response::Message::<Ping,Pong>::Response {
                                request_id, response
                            },
                    })) => {
                        info!("Heartbeat protocol: Received a Pong response for request {} from {}", request_id, peer);
                        if let Some(_) = active_manual_pings.take(&request_id) {
                            debug!("Processing manual ping response from peer {}", peer);
                            match manual_ping_responds.record_pong((peer, Ok(response.0))).await {
                                Ok(_) => {},
                                Err(e) => {
                                    error!("Manual ping mechanism could not be updated with pong messages: {}", e);
                                }
                            }
                        } else {
                            match heartbeat_responds.record_pong((peer, Ok(response.0))).await {
                                Ok(_) => {},
                                Err(e) => {
                                    error!("Heartbeat mechanism could not be updated with pong messages: {}", e);
                                }
                            }
                        }
                    },
                    SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::Heartbeat(libp2p_request_response::Event::<Ping,Pong>::OutboundFailure {
                        peer, request_id, error,
                    })) => {
                        info!("Heartbeat protocol: Failed to send a Ping message {} to {} with an error: {}", request_id, peer, error);
                        match heartbeat_responds.record_pong((peer, Err(()))).await {
                            Ok(_) => {},
                            Err(e) => {
                                error!("Heartbeat mechanism could not be updated with pong messages: {}", e);
                            }
                        }
                    },
                    SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::Heartbeat(libp2p_request_response::Event::<Ping,Pong>::InboundFailure {..}))
                    | SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::Heartbeat(libp2p_request_response::Event::<Ping,Pong>::ResponseSent {..})) => {
                        // debug!("Discarded messages not relevant for the protocol!");
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
