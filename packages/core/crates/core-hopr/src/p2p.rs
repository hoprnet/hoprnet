use std::sync::Arc;

use async_lock::RwLock;
use futures::{channel::mpsc::Receiver, select, StreamExt};
use futures_concurrency::stream::Merge;

use core_network::network::{Network, NetworkEvent, PeerOrigin};
pub use core_p2p::{api, libp2p_identity};
use core_p2p::{libp2p_request_response, libp2p_swarm::SwarmEvent, HoprNetworkBehaviorEvent, Ping, Pong};
use core_packet::interaction::{AckProcessed, AcknowledgementInteraction, MsgProcessed, PacketInteraction};
use core_types::acknowledgement::Acknowledgement;
use utils_log::{debug, error, info};

use crate::{adaptors::indexer::IndexerProcessed, PeerId};

#[derive(Debug)]
pub enum Inputs {
    Heartbeat(api::HeartbeatChallenge),
    ManualPing(api::ManualPingChallenge),
    NetworkUpdate(NetworkEvent),
    Message(MsgProcessed),
    Acknowledgement(AckProcessed),
    Indexer(IndexerProcessed),
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

impl From<IndexerProcessed> for Inputs {
    fn from(value: IndexerProcessed) -> Self {
        Self::Indexer(value)
    }
}

/// Main p2p loop that will instantiate a new libp2p::Swarm instance and setup listening and reacting pipelines
/// running in a neverending loop future.
///
/// The function represents the entirety of the business logic of the hopr daemon related to core operations.
///
/// This future can only be resolved by an unrecoverable error or a panic.
pub(crate) async fn p2p_loop(
    me: libp2p_identity::Keypair,
    network: Arc<RwLock<Network<crate::adaptors::network::ExternalNetworkInteractions>>>,
    network_update_input: Receiver<NetworkEvent>,
    indexer_update_input: Receiver<IndexerProcessed>,
    ack_interactions: AcknowledgementInteraction,
    pkt_interactions: PacketInteraction,
    heartbeat_requests: api::HeartbeatRequester,
    heartbeat_responds: api::HeartbeatResponder,
    manual_ping_requests: api::ManualPingRequester,
    manual_ping_responds: api::HeartbeatResponder,
    my_multiaddresses: Vec<multiaddr::Multiaddr>,
) {
    let mut swarm = core_p2p::build_p2p_network(me);

    let mut valid_mas: Vec<multiaddr::Multiaddr> = vec![];
    for multiaddress in my_multiaddresses.iter() {
        // NOTE: Due to lack of STUN the passed in multiaddresses are believed to be correct after
        // the first successful listen. Relevant for Providence, but not beyond.
        if valid_mas.len() > 0 {
            valid_mas.push(multiaddress.clone());
            continue;
        }

        match swarm.listen_on(multiaddress.clone()) {
            Ok(_) => {
                valid_mas.push(multiaddress.clone());
                swarm.add_external_address(multiaddress.clone());
            }
            Err(e) => {
                error!("Failed to listen_on using the multiaddress '{}': {}", multiaddress, e);
            }
        }
    }

    info!("Registering own external multiaddresses: {:?}", valid_mas);
    network
        .write()
        .await
        .store_peer_multiaddresses(swarm.local_peer_id(), valid_mas);

    let mut heartbeat_responds = heartbeat_responds;
    let mut manual_ping_responds = manual_ping_responds;

    let mut ack_writer = ack_interactions.writer();
    let mut pkt_writer = pkt_interactions.writer();

    let mut active_manual_pings: std::collections::HashSet<libp2p_request_response::RequestId> =
        std::collections::HashSet::new();
    let mut allowed_peers: std::collections::HashSet<PeerId> = std::collections::HashSet::new();

    let mut inputs = (
        heartbeat_requests.map(Inputs::Heartbeat),
        manual_ping_requests.map(Inputs::ManualPing),
        network_update_input.map(Inputs::NetworkUpdate),
        ack_interactions.map(Inputs::Acknowledgement),
        pkt_interactions.map(Inputs::Message),
        indexer_update_input.map(Inputs::Indexer),
    )
        .merge()
        .fuse();

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
                    MsgProcessed::Send(peer, octets) => {
                        let _request_id = swarm.behaviour_mut().msg.send_request(&peer, octets);
                    },
                    MsgProcessed::Forward(peer, octets) => {
                        let _request_id = swarm.behaviour_mut().msg.send_request(&peer, octets);
                    }
                },
                Inputs::Indexer(task) => match task {
                    IndexerProcessed::Allow(peer) => {
                        allowed_peers.insert(peer);
                    },
                    IndexerProcessed::Ban(peer) => {
                        allowed_peers.remove(&peer);

                        if swarm.is_connected(&peer) {
                            match swarm.disconnect_peer_id(peer) {
                                Ok(_) => debug!("Peer '{peer}' disconnected"),
                                Err(e) => error!("Failed to disconnect peer '{peer}': {:?}", e)
                            }
                        }
                    },
                    IndexerProcessed::Announce(peer, multiaddresses) => {
                        for multiaddress in multiaddresses.iter() {
                            if !swarm.is_connected(&peer) {
                                match swarm.dial(multiaddress.clone()) {
                                    Ok(_) => {
                                        swarm.behaviour_mut().heartbeat.add_address(&peer, multiaddress.clone());
                                        swarm.behaviour_mut().msg.add_address(&peer, multiaddress.clone());
                                        swarm.behaviour_mut().ack.add_address(&peer, multiaddress.clone());
                                    },
                                    Err(e) => {
                                        error!("Failed to dial an announced peer '{}': {}, skipping the address", &peer, e);
                                    }
                                }
                            }
                        }

                        // TODO: awaiting in this loop is a malpractice, but this behavior will be handled by STUN later
                        {
                            let mut net = network.write().await;
                            net.store_peer_multiaddresses(&peer, multiaddresses);
                            if &peer != swarm.local_peer_id() {
                                net.add(&peer, PeerOrigin::Initialization)
                            }
                        }

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
                },
                SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::Message(libp2p_request_response::Event::<Box<[u8]>, ()>::OutboundFailure {
                    peer, error, request_id
                })) => {
                    error!("Message protocol: Failed to send a message (#{}) to peer {} with an error: {}", request_id, peer, error);
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
                // heartbeat protocol
                SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::Heartbeat(libp2p_request_response::Event::<Ping,Pong>::Message {
                    peer,
                    message:
                    libp2p_request_response::Message::<Ping,Pong>::Request {
                        request_id, request, channel
                    },
                })) => {
                    info!("Received a Ping request {} from {}", request_id, peer);
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
                    info!("Received a Pong response from {} (#{}) ", peer, request_id);
                    if active_manual_pings.take(&request_id).is_some() {
                        debug!("Processing manual ping response from peer {}", peer);
                        match manual_ping_responds.record_pong((peer, Ok(response.0))).await {
                            Ok(_) => {},
                            Err(e) => {
                                error!("Manual ping mechanism could not be updated with pong messages: {}", e);
                            }
                        }
                    } else {
                        debug!("Processing heartbeat ping response from peer {}", peer);
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
                SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::Heartbeat(libp2p_request_response::Event::<Ping,Pong>::InboundFailure {
                    peer, request_id, error})) => {
                    debug!("Heartbeat protocol: Encountered inbound failure for peer {} (#{}): {}", peer, request_id, error)
                }
                SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::Heartbeat(libp2p_request_response::Event::<Ping,Pong>::ResponseSent {..})) => {
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
                } => {
                    debug!("Connection established with {:?}", peer_id);
                    let _ = &allowed_peers.iter().for_each(|p| {
                        debug!("ALLOWED PEERS: {:?}", p)
                    });
                    if allowed_peers.contains(&peer_id) {
                        if ! (*network.read().await).has(&peer_id) {
                            (*network.write().await).add(&peer_id, PeerOrigin::IncomingConnection)
                        }
                    } else {
                    debug!("DISCONNECTION PEER ID {:?}", peer_id);
                       let _ = swarm.disconnect_peer_id(peer_id);
                    }
                },
                SwarmEvent::ConnectionClosed {
                    peer_id,
                    cause,
                    ..
                    // connection_id,
                    // endpoint,
                    // num_established,
                } => {
                    debug!("Connection closed for peer {:?}: {:?}", peer_id, cause)
                },
                SwarmEvent::IncomingConnection {
                    connection_id,
                    local_addr,
                    send_back_addr,
                } => {
                    debug!("Incoming connection at {local_addr} from {send_back_addr} ({:?})", connection_id);
                },
                SwarmEvent::IncomingConnectionError {
                    local_addr,
                    ..
                    // connection_id,
                    // send_back_addr,
                    // error,
                } => {
                    debug!("Incoming connection error on {:?}", local_addr)
                },
                SwarmEvent::OutgoingConnectionError {
                    connection_id,
                    error,
                    peer_id
                } => {
                    error!("Outgoing connection error for peer '{:?}' ({:?}): {}", peer_id, connection_id, error)
                },
                SwarmEvent::NewListenAddr {
                    listener_id,
                    ..
                    // address,
                } => {
                    debug!("New listen addr {:?}", listener_id)
                },
                SwarmEvent::ExpiredListenAddr {
                    listener_id,
                    ..
                    // address,
                } => {
                    debug!("Expired listen addr {:?}", listener_id)
                },
                SwarmEvent::ListenerClosed {
                    listener_id,
                    ..
                    // addresses,
                    // reason,
                } => {
                    debug!("Listener closed {:?}", listener_id)
                },
                SwarmEvent::ListenerError {
                    listener_id,
                    error,
                } => {
                    debug!("Listener error for the id {:?}: {}", listener_id, error)
                },
                SwarmEvent::Dialing {
                    peer_id,
                    connection_id,
                } => {
                    debug!("Dialing peer {:?}, connection id: {:?}", peer_id, connection_id)
                },
            }
        }
    }
}
