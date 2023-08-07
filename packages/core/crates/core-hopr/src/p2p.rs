use std::sync::Arc;

use async_lock::RwLock;
use core_network::network::{Network, NetworkEvent};
use futures::{
    select, StreamExt,
    channel::mpsc::Receiver
};
use futures_concurrency::stream::Merge;

pub use core_p2p::{libp2p_identity, api};
use core_p2p::{
    HoprSwarm, HoprNetworkBehaviorEvent,
    Ping, Pong,
    libp2p_request_response, libp2p_swarm::SwarmEvent
};
use utils_log::{debug, info, error};


#[derive(Debug, Clone, PartialEq)]
pub enum Inputs {
    Heartbeat(api::HeartbeatChallenge),
    ManualPing(api::ManualPingChallenge),
    NetworkUpdate(NetworkEvent),
    // Message(String),                 // TODO: This should hold the `Packet` object
    // Acknowledgement(String),         // TODO: This should hold the `Acknowledgment` object
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

impl From<NetworkEvent> for Inputs {
    fn from(value: NetworkEvent) -> Self {
        Self::NetworkUpdate(value)
    }
}


/// Main p2p loop that will instantiate a new libp2p::Swarm instance and setup listeining and reacting pipelines
/// running in a neverending loop future.
/// 
/// This future can only be resolved by an error or a panic.
pub(crate) async fn p2p_loop(me: libp2p_identity::Keypair,
    network: Arc<RwLock<Network<crate::adaptors::network::ExternalNetworkInteractions>>>,
    network_update_input: Receiver<NetworkEvent>,
    heartbeat_requests: api::HeartbeatRequester,
    heartbeat_responds: api::HeartbeatResponder,
    manual_ping_requests: api::ManualPingRequester,
    manual_ping_responds: api::HeartbeatResponder)
{
    let mut swarm = core_p2p::build_p2p_network(me);
    let mut heartbeat_responds = heartbeat_responds;
    let mut manual_ping_responds = manual_ping_responds;
    
    let mut active_manual_pings: std::collections::HashSet<libp2p_request_response::RequestId> = std::collections::HashSet::new();
    
    let mut inputs = (
        heartbeat_requests.map(Inputs::Heartbeat),
        manual_ping_requests.map(Inputs::ManualPing),
        network_update_input.map(Inputs::NetworkUpdate)
    ).merge().fuse();
    
    loop {
        select! {
            input = inputs.select_next_some() => match input {
                Inputs::Heartbeat(api::HeartbeatChallenge(peer, challenge)) => {
                    swarm.behaviour_mut().heartbeat.send_request(&peer, Ping(challenge));
                },
                Inputs::ManualPing(api::ManualPingChallenge(peer, challenge)) => {
                    let req_id = swarm.behaviour_mut().heartbeat.send_request(&peer, Ping(challenge));
                    active_manual_pings.insert(req_id);
                },
                Inputs::NetworkUpdate(event) => match event {
                    NetworkEvent::CloseConnection(peer) => {
                        if swarm.is_connected(&peer) {
                            let _ = swarm.disconnect_peer_id(peer);
                        }
                    },
                    NetworkEvent::PeerOffline(_peer) => {
                        // TODO: this functionality may not be needed after swtich to rust-libp2p
                    },
                    NetworkEvent::Register(peer, origin) => {
                        let mut writer = network.write().await;
                        (*writer).add(&peer, origin)
                    },
                    NetworkEvent::Unregister(peer) => {
                        let mut writer = network.write().await;
                        (*writer).remove(&peer)
                    },
                }
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
    };
}