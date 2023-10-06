use std::sync::Arc;

use crate::{adaptors::indexer::IndexerProcessed, PeerId};
use async_lock::RwLock;
use core_crypto::types::HalfKeyChallenge;
use core_network::network::{Network, NetworkEvent, PeerOrigin};
pub use core_p2p::{api, libp2p_identity};
use core_p2p::{
    libp2p_request_response::{self, ResponseChannel},
    libp2p_swarm::SwarmEvent,
    HoprNetworkBehaviorEvent, Ping, Pong,
};
use core_protocol::{
    ack::{
        config::AckProtocolConfig,
        processor::{AckProcessed, AcknowledgementInteraction, Reply},
    },
    heartbeat::config::HeartbeatProtocolConfig,
    msg::{
        config::MsgProtocolConfig,
        processor::{MsgProcessed, PacketInteraction},
    },
    ticket_aggregation::{
        config::TicketAggregationProtocolConfig,
        processor::{TicketAggregationFinalizer, TicketAggregationInteraction, TicketAggregationProcessed},
    },
};
use core_types::{
    acknowledgement::{AcknowledgedTicket, Acknowledgement},
    channels::Ticket,
    protocol::ApplicationData,
};
use futures::{
    channel::mpsc::{Receiver, Sender, UnboundedSender},
    select, StreamExt,
};
use futures_concurrency::stream::Merge;
use libp2p::request_response::RequestId;
use std::collections::{HashMap, HashSet};
use utils_log::{debug, error, info};

#[derive(Debug)]
pub enum Inputs {
    Heartbeat(api::HeartbeatChallenge),
    ManualPing(api::ManualPingChallenge),
    NetworkUpdate(NetworkEvent),
    Message(MsgProcessed),
    TicketAggregation(TicketAggregationProcessed<ResponseChannel<Result<Ticket, String>>, RequestId>),
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

impl From<TicketAggregationProcessed<ResponseChannel<Result<Ticket, String>>, RequestId>> for Inputs {
    fn from(value: TicketAggregationProcessed<ResponseChannel<Result<Ticket, String>>, RequestId>) -> Self {
        Self::TicketAggregation(value)
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
    ticket_aggregation_interactions: TicketAggregationInteraction<ResponseChannel<Result<Ticket, String>>, RequestId>,
    heartbeat_requests: api::HeartbeatRequester,
    heartbeat_responds: api::HeartbeatResponder,
    manual_ping_requests: api::ManualPingRequester,
    manual_ping_responds: api::HeartbeatResponder,
    my_multiaddresses: Vec<multiaddr::Multiaddr>,
    ack_proto_cfg: AckProtocolConfig,
    heartbeat_proto_cfg: HeartbeatProtocolConfig,
    msg_proto_cfg: MsgProtocolConfig,
    ticket_aggregation_proto_cfg: TicketAggregationProtocolConfig,
    mut on_final_packet: Sender<ApplicationData>,
    on_acknowledgement: UnboundedSender<HalfKeyChallenge>,
    on_acknowledged_ticket: UnboundedSender<AcknowledgedTicket>,
) {
    let mut swarm = core_p2p::build_p2p_network(
        me,
        ack_proto_cfg,
        heartbeat_proto_cfg,
        msg_proto_cfg,
        ticket_aggregation_proto_cfg,
    );

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
    let mut aggregation_writer = ticket_aggregation_interactions.writer();

    let mut active_manual_pings: HashSet<libp2p_request_response::RequestId> = HashSet::new();
    let mut active_aggregation_requests: HashMap<libp2p_request_response::RequestId, TicketAggregationFinalizer> =
        HashMap::new();

    let mut allowed_peers: HashSet<PeerId> = HashSet::new();

    let mut inputs = (
        heartbeat_requests.map(Inputs::Heartbeat),
        manual_ping_requests.map(Inputs::ManualPing),
        network_update_input.map(Inputs::NetworkUpdate),
        ack_interactions.map(Inputs::Acknowledgement),
        pkt_interactions.map(Inputs::Message),
        ticket_aggregation_interactions.map(Inputs::TicketAggregation),
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
                    debug!("Executing manual ping to peer '{}'", peer);
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
                        // NOTE: this functionality is not needed after swtich to rust-libp2p
                    },
                    NetworkEvent::Register(peer, origin, metadata) => {
                        let mut writer = network.write().await;
                        (*writer).add_with_metadata(&peer, origin, metadata)
                    },
                    NetworkEvent::Unregister(peer) => {
                        let mut writer = network.write().await;
                        (*writer).remove(&peer)
                    },
                },
                Inputs::Acknowledgement(task) => match task {
                    AckProcessed::Receive(peer, reply) => {
                        debug!("Received an acknowledgement from {peer}");
                        if let Ok(reply) = reply {
                            match reply {
                                Reply::Sender(half_key_challenge) => {
                                    if let Err(e) = on_acknowledgement.unbounded_send(half_key_challenge) {
                                        error!("failed to emit received acknowledgement: {e}")
                                    }
                                },
                                Reply::RelayerWinning(acknowledged_ticket) => {
                                    if let Err(e) = on_acknowledged_ticket.unbounded_send(acknowledged_ticket) {
                                        error!("failed to emit acknowledged ticket: {e}");
                                    }
                                }
                                Reply::RelayerLosing => {}
                            }
                        }
                    },
                    AckProcessed::Send(peer, ack) => {
                        let _request_id = swarm.behaviour_mut().ack.send_request(&peer, ack);
                    }
                }
                Inputs::Message(task) => match task {
                    MsgProcessed::Receive(peer, data, ack) => {
                        debug!("Received packet from peer: {peer}");
                        if let Err(e) = on_final_packet.try_send(data) {
                            error!("Failed to store a received message in the inbox: {}", e);
                        }

                        if let Some(ack) = ack {
                            if let Err(e) = ack_writer.send_acknowledgement(peer, ack) {
                                error!("Failed to acknowledge the received final packet: {e}");
                            }
                        }
                    },
                    MsgProcessed::Send(peer, octets) => {
                        let _request_id = swarm.behaviour_mut().msg.send_request(&peer, octets);
                    },
                    MsgProcessed::Forward(peer, octets, previous_peer, ack) => {
                        let _request_id = swarm.behaviour_mut().msg.send_request(&peer, octets);
                        if let Some(ack) = ack {
                            if let Err(e) = ack_writer.send_acknowledgement(previous_peer, ack) {
                                error!("failed to acknowledge relayed packet: {e}");
                            }
                        }
                    }
                },
                Inputs::TicketAggregation(task) => match task {
                    TicketAggregationProcessed::Send(peer, acked_tickets, finalizer) => {
                        let request_id = swarm.behaviour_mut().ticket_aggregation.send_request(&peer, acked_tickets);
                        error!("Ticket aggregation: Sent request (#{}) to {}", request_id, peer);
                        active_aggregation_requests.insert(request_id, finalizer);
                    },
                    TicketAggregationProcessed::Reply(peer, ticket, response) => {
                        if let Err(_) = swarm.behaviour_mut().ticket_aggregation.send_response(response, ticket) {
                            error!("Ticket aggregation: Failed send reply to {}", peer);
                        }
                    },
                    TicketAggregationProcessed::Receive(_peer, _acked_ticket, request) => {
                        // TODO: uncomment once strategies need to get the value
                        // if let Err(e) = on_acknowledged_ticket.unbounded_send(acked_ticket) {
                        //     error!("failed to emit acknowledged aggregated ticket: {e}");
                        // }

                        match active_aggregation_requests.remove(&request) {
                            Some(finalizer) => finalizer.finalize(),
                            None => {
                                debug!("Ticket aggregation: response already handled")
                            }
                        }
                    }
                },
                Inputs::Indexer(task) => match task {
                    IndexerProcessed::Allow(peer) => {
                        let _ = allowed_peers.insert(peer);
                    }
                    IndexerProcessed::Ban(peer) => {
                        allowed_peers.remove(&peer);

                        if swarm.is_connected(&peer) {
                            match swarm.disconnect_peer_id(peer) {
                                Ok(_) => debug!("Peer '{peer}' disconnected on network registry update"),
                                Err(e) => error!("Failed to disconnect peer '{peer}' on network registry update: {:?}", e)
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
                                        swarm.behaviour_mut().ticket_aggregation.add_address(&peer, multiaddress.clone());
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
                // ticket aggregation protocol
                SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::TicketAggregation(libp2p_request_response::Event::<Vec<AcknowledgedTicket>, std::result::Result<Ticket,String>>::Message {
                    peer,
                    message:
                    libp2p_request_response::Message::<Vec<AcknowledgedTicket>, std::result::Result<Ticket,String>>::Request {
                        request_id, request, channel
                    },
                })) => {
                    info!("Ticket aggregation protocol: Received an aggregation request {} from {}", request_id, peer);
                    if let Err(e) = aggregation_writer.receive_aggregation_request(peer, request, channel) {
                        debug!("Aggregation protocol: Failed to aggregate tickets for {} with an error: {}", peer, e);
                    }
                },
                SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::TicketAggregation(libp2p_request_response::Event::<Vec<AcknowledgedTicket>, std::result::Result<Ticket,String>>::Message {
                    peer,
                    message:
                    libp2p_request_response::Message::<Vec<AcknowledgedTicket>, std::result::Result<Ticket,String>>::Response {
                        request_id, response
                    },
                })) => {
                    if let Err(e) = aggregation_writer.receive_ticket(peer, response, request_id) {
                        debug!("Aggregation protocol: Error while handling aggregated ticket from {}, error: {}", peer, e);
                    }
                },
                SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::TicketAggregation(libp2p_request_response::Event::<Vec<AcknowledgedTicket>, std::result::Result<Ticket,String>>::OutboundFailure {
                    peer, request_id, error,
                })) => {
                    info!("Ticket aggregation protocol: Failed to send a Ping message {} to {} with an error: {}", request_id, peer, error);
                },
                SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::TicketAggregation(libp2p_request_response::Event::<Vec<AcknowledgedTicket>, std::result::Result<Ticket,String>>::InboundFailure {
                    peer, request_id, error})) => {
                    debug!("Ticket aggregation protocol: Encountered inbound failure for peer {} (#{}): {}", peer, request_id, error)
                }
                SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::TicketAggregation(libp2p_request_response::Event::<Vec<AcknowledgedTicket>, std::result::Result<Ticket,String>>::ResponseSent {..})) => {
                    // debug!("Ticket aggregation protocol: Discarded messages not relevant for the protocol!");
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
                    match swarm.behaviour_mut().heartbeat.send_response(channel, Pong(challenge_response, crate::constants::APP_VERSION.to_owned())) {
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
                        match manual_ping_responds.record_pong((peer, Ok((response.0, response.1)))).await {
                            Ok(_) => {},
                            Err(e) => {
                                error!("Manual ping mechanism could not be updated with pong messages: {}", e);
                            }
                        }
                    } else {
                        debug!("Processing heartbeat ping response from peer {}", peer);
                        match heartbeat_responds.record_pong((peer, Ok((response.0, response.1)))).await {
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
                    if allowed_peers.contains(&peer_id) {
                        if ! (*network.read().await).has(&peer_id) {
                            (*network.write().await).add(&peer_id, PeerOrigin::IncomingConnection)
                        }
                    } else {
                        debug!("DISCONNECTION (based on network registry) for PEER ID {:?}", peer_id);
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
