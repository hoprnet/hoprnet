use std::sync::Arc;

use async_lock::RwLock;
use core_crypto::types::HalfKeyChallenge;
use core_network::network::{Network, NetworkEvent};
use core_types::acknowledgement::Acknowledgement;
use futures::{
    select, StreamExt,
    channel::mpsc::Receiver
};
use futures_concurrency::stream::Merge;

use core_packet::interaction::{AckProcessed, AcknowledgementInteraction, MsgProcessed, PacketInteraction, PacketSendFinalizer};
pub use core_p2p::{libp2p_identity, api};
use core_p2p::{
    HoprNetworkBehaviorEvent,
    Ping, Pong,
    libp2p_request_response, libp2p_swarm::SwarmEvent
};
use utils_log::{debug, info, error};


#[derive(Debug)]
pub enum Inputs {
    Heartbeat(api::HeartbeatChallenge),
    ManualPing(api::ManualPingChallenge),
    NetworkUpdate(NetworkEvent),
    Message(MsgProcessed),
    Acknowledgement(AckProcessed),
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

impl From<AckProcessed> for Inputs {
    fn from(value: AckProcessed) -> Self {
        Self::Acknowledgement(value)
    }
}

impl From<MsgProcessed> for Inputs {
    fn from(value: MsgProcessed) -> Self {
        Self::Message(value)
    }
}



/// Main p2p loop that will instantiate a new libp2p::Swarm instance and setup listening and reacting pipelines
/// running in a neverending loop future.
/// 
/// This future can only be resolved by an error or a panic.
pub(crate) async fn p2p_loop(me: libp2p_identity::Keypair,
    network: Arc<RwLock<Network<crate::adaptors::network::ExternalNetworkInteractions>>>,
    network_update_input: Receiver<NetworkEvent>,
    ack_interactions: AcknowledgementInteraction,
    pkt_interactions: PacketInteraction,
    heartbeat_requests: api::HeartbeatRequester,
    heartbeat_responds: api::HeartbeatResponder,
    manual_ping_requests: api::ManualPingRequester,
    manual_ping_responds: api::HeartbeatResponder)
{
    let mut swarm = core_p2p::build_p2p_network(me);
    let mut heartbeat_responds = heartbeat_responds;
    let mut manual_ping_responds = manual_ping_responds;

    let mut ack_writer = ack_interactions.writer();
    let mut pkt_writer = pkt_interactions.writer();

    let mut active_manual_pings: std::collections::HashSet<libp2p_request_response::RequestId> = std::collections::HashSet::new();
    let mut active_sent_packets: std::collections::HashMap<libp2p_request_response::RequestId, PacketSendFinalizer> = std::collections::HashMap::new();


    let mut inputs = (
        heartbeat_requests.map(Inputs::Heartbeat),
        manual_ping_requests.map(Inputs::ManualPing),
        network_update_input.map(Inputs::NetworkUpdate),
        ack_interactions.map(Inputs::Acknowledgement),
        pkt_interactions.map(Inputs::Message),
    ).merge().fuse();
    
    // NOTE: this should be changed to a merged stream as well, maybe `SwarmEvent<HoprNetworkBehaviorEvent>`
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
                },
                Inputs::Acknowledgement(task) => match task {
                    AckProcessed::Receive(peer, _result) => {
                        debug!("Nothing needs to be done here, as long as the ack interactions emits the received acknowledgement for peer '{peer}'")
                    },
                    AckProcessed::Send(peer, ack) => {
                        let _request_id = swarm.behaviour_mut().ack.send_request(&peer, ack);
                    }
                }
                Inputs::Message(task) => match task {
                    MsgProcessed::Receive(peer, _octets) => {
                        debug!("Nothing needs to be done here, as long as the packet interactions emit the received packet from peer: {peer}")
                    },
                    MsgProcessed::Send(peer, octets, finalizer) => {
                        let request_id = swarm.behaviour_mut().msg.send_request(&peer, octets);
                        active_sent_packets.insert(request_id, finalizer);
                    },
                    MsgProcessed::Forward(peer, octets) => {
                        let _request_id = swarm.behaviour_mut().msg.send_request(&peer, octets);
                    }
                }
            },
            event = swarm.select_next_some() => match event {
                // ---------------
                // msg protocol
                SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::Message(libp2p_request_response::Event::<Box<[u8]>, ()>::Message {
                    peer,
                    message:
                    libp2p_request_response::Message::<Box<[u8]>, ()>::Request {
                        request_id, request, channel
                    },
                })) => {
                    debug!("Message protocol: Received a message from {}", &peer);

                    if let Err(e) = pkt_writer.receive_packet(request, peer.clone()) {
                        error!("Message protocol: Failed to process a message from {}: {} (#{})", &peer, e, request_id);
                    };

                    if let Err(_) = swarm.behaviour_mut().msg.send_response(channel, ()) {
                        error!("Message protocol: Failed to send a response to {}, likely a timeout", &peer);
                    };
                },
                SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::Message(libp2p_request_response::Event::<Box<[u8]>, ()>::Message {
                    peer,
                    message:
                    libp2p_request_response::Message::<Box<[u8]>, ()>::Response {
                        request_id, ..
                    },
                })) => {
                    debug!("Message protocol: Received a response for sending message with id {} from {}", &request_id, &peer);
                    if let Some(finalizer) = active_sent_packets.remove(&request_id) {
                        finalizer.send();
                    }
                },
                SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::Message(libp2p_request_response::Event::<Box<[u8]>, ()>::OutboundFailure {
                    peer, error, request_id
                })) => {
                    error!("Message protocol: Failed to send a message {} with an error: {}", peer, error);
                    active_sent_packets.remove(&request_id);
                },
                SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::Message(libp2p_request_response::Event::<Box<[u8]>, ()>::InboundFailure {..}))
                | SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::Message(libp2p_request_response::Event::<Box<[u8]>, ()>::ResponseSent {..})) => {
                    // debug!("Discarded messages not relevant for the protocol!");
                },
                // ---------------
                // ack protocol
                SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::Acknowledgement(libp2p_request_response::Event::<Acknowledgement,()>::Message {
                    peer,
                    message:
                    libp2p_request_response::Message::<Acknowledgement,()>::Request {
                        request_id, request, channel
                    },
                })) => {
                    debug!("Ack protocol: Received an acknowledgment from {}", &peer);

                    if let Err(e) = ack_writer.receive_acknowledgement(peer.clone(), request) {
                        error!("Ack protocol: Failed to process an acknowledgement from {}: {} (#{})", &peer, e, request_id);
                    };

                    if let Err(_) = swarm.behaviour_mut().ack.send_response(channel, ()) {
                        error!("Ack protocol: Failed to send a response to {}, likely a timeout", &peer);
                    };
                },
                SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::Acknowledgement(libp2p_request_response::Event::<Acknowledgement,()>::Message {
                    peer,
                    message:
                    libp2p_request_response::Message::<Acknowledgement,()>::Response {
                        request_id, ..
                    },
                })) => {
                    debug!("Ack protocol: Received a response for sending message with id {} from {}", request_id, peer);
                },
                SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::Acknowledgement(libp2p_request_response::Event::<Acknowledgement,()>::OutboundFailure {
                    peer, error, ..
                })) => {
                    error!("Ack protocol: Failed to send an acknowledgement {} with an error: {}", peer, error);
                },
                SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::Acknowledgement(libp2p_request_response::Event::<Acknowledgement,()>::InboundFailure {..}))
                | SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::Acknowledgement(libp2p_request_response::Event::<Acknowledgement,()>::ResponseSent {..})) => {
                    // debug!("Discarded messages not relevant for the protocol!");
                },
                // --------------
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
                SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::KeepAlive(_)) => {
                    // debug!("Keep alive tick to make sure the loop never ends")
                }
                SwarmEvent::ConnectionEstablished {
                    peer_id,
                    ..
                    // connection_id,
                    // endpoint,
                    // num_established,
                    // concurrent_dial_errors,
                    // established_in,
                } => {debug!("Connection established with {:?}", peer_id)},
                SwarmEvent::ConnectionClosed {
                    peer_id,
                    ..
                    // connection_id,
                    // endpoint,
                    // num_established,
                    // cause,
                } => {debug!("Connection closed for peer {:?}", peer_id)},
                SwarmEvent::IncomingConnection {
                    connection_id,
                    local_addr,
                    send_back_addr,
                } => {debug!("Incoming connection on {:?} from {:?} (conn_id: {:?})", local_addr, send_back_addr, connection_id)},
                SwarmEvent::IncomingConnectionError {
                    local_addr,
                    ..
                    // connection_id,
                    // send_back_addr,
                    // error,
                } => {debug!("Incoming connection error on {:?}", local_addr)},
                SwarmEvent::OutgoingConnectionError {
                    connection_id,
                    error,
                    ..
                    // peer_id
                } => {debug!("Outgoing connection error {:?}: {}", connection_id, error)},
                SwarmEvent::NewListenAddr {
                    listener_id,
                    ..
                    // address,
                } => {debug!("New listen addr {:?}", listener_id)},
                SwarmEvent::ExpiredListenAddr {
                    listener_id,
                    ..
                    // address,
                } => {debug!("Expired listen addr {:?}", listener_id)},
                SwarmEvent::ListenerClosed {
                    listener_id,
                    ..
                    // addresses,
                    // reason,
                } => {debug!("Listener closed {:?}", listener_id)},
                SwarmEvent::ListenerError {
                    listener_id,
                    error,
                } => {debug!("Listener error for the id {:?}: {}", listener_id, error)},
                SwarmEvent::Dialing {
                    peer_id,
                    connection_id,
                } => {debug!("Dialing peer {:?}, connection id: {:?}", peer_id, connection_id)},
            }
        }
    };
}