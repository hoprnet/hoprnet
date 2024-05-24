use futures::{channel::mpsc::UnboundedSender, pin_mut, select, StreamExt};
use futures_concurrency::stream::Merge;
use hopr_crypto_types::keypairs::OffchainKeypair;
use hopr_internal_types::prelude::*;
use libp2p::request_response::OutboundRequestId;
use std::{
    collections::{HashMap, HashSet},
    net::Ipv4Addr,
    sync::Arc,
};
use tracing::{debug, error, info, trace, warn};

#[cfg(feature = "runtime-async-std")]
use async_std::task::spawn;

#[cfg(feature = "runtime-tokio")]
use tokio::task::spawn;

use core_network::{
    network::{Network, NetworkTriggeredEvent, PeerOrigin},
    HoprDbPeersOperations,
};
pub use core_p2p::api;
use core_p2p::{
    libp2p::{request_response::ResponseChannel, swarm::SwarmEvent},
    HoprNetworkBehavior, HoprNetworkBehaviorEvent, Ping, Pong,
};
use core_protocol::{
    ack::processor::{AckProcessed, AckResult, AcknowledgementInteraction},
    config::ProtocolConfig,
    msg::processor::{MsgProcessed, PacketInteraction},
    ticket_aggregation::processor::{
        TicketAggregationFinalizer, TicketAggregationInteraction, TicketAggregationProcessed,
    },
};

use crate::{processes::indexer::PeerTransportEvent, PeerId, TransportOutput};

#[cfg(all(feature = "prometheus", not(test)))]
use hopr_metrics::metrics::SimpleGauge;

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_TRANSPORT_P2P_OPEN_CONNECTION_COUNT: SimpleGauge = SimpleGauge::new(
        "hopr_transport_p2p_opened_connection_count",
        "Number of currently open connections"
    ).unwrap();
}

/// Composition of all inputs allowing to produce a single stream of
/// input events passed into the swarm processing logic.
#[allow(clippy::large_enum_variant)] // TODO: refactor the large types used in the enum
#[derive(Debug)]
pub enum Inputs {
    Heartbeat(api::HeartbeatChallenge),
    ManualPing(api::ManualPingChallenge),
    NetworkUpdate(NetworkTriggeredEvent),
    Message(MsgProcessed),
    TicketAggregation(TicketAggregationProcessed<ResponseChannel<Result<Ticket, String>>, OutboundRequestId>),
    Acknowledgement(AckProcessed),
    Indexer(PeerTransportEvent),
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

impl From<NetworkTriggeredEvent> for Inputs {
    fn from(value: NetworkTriggeredEvent) -> Self {
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

impl From<TicketAggregationProcessed<ResponseChannel<Result<Ticket, String>>, OutboundRequestId>> for Inputs {
    fn from(value: TicketAggregationProcessed<ResponseChannel<Result<Ticket, String>>, OutboundRequestId>) -> Self {
        Self::TicketAggregation(value)
    }
}

impl From<PeerTransportEvent> for Inputs {
    fn from(value: PeerTransportEvent) -> Self {
        Self::Indexer(value)
    }
}

use hopr_internal_types::legacy;
use std::net::ToSocketAddrs;

/// Replaces the IPv4 and IPv6 from the network layer with a unspecified interface in any multiaddress.
fn replace_transport_with_unspecified(ma: &multiaddr::Multiaddr) -> crate::errors::Result<multiaddr::Multiaddr> {
    let mut out = multiaddr::Multiaddr::empty();

    for proto in ma.iter() {
        match proto {
            multiaddr::Protocol::Ip4(_) => out.push(std::net::IpAddr::V4(Ipv4Addr::UNSPECIFIED).into()),
            multiaddr::Protocol::Ip6(_) => out.push(std::net::IpAddr::V6(std::net::Ipv6Addr::UNSPECIFIED).into()),
            _ => out.push(proto),
        }
    }

    Ok(out)
}

/// Resolves the DNS parts of a multiaddress and replaces it with the resolved IP address.
fn resolve_dns_if_any(ma: &multiaddr::Multiaddr) -> crate::errors::Result<multiaddr::Multiaddr> {
    let mut out = multiaddr::Multiaddr::empty();

    for proto in ma.iter() {
        match proto {
            multiaddr::Protocol::Dns4(domain) => {
                let ip = format!("{domain}:443") // dummy port, irrevelant at this point
                    .to_socket_addrs()
                    .map_err(|e| crate::errors::HoprTransportError::Api(e.to_string()))?
                    .filter(|sa| sa.is_ipv4())
                    .collect::<Vec<_>>()
                    .first()
                    .ok_or(crate::errors::HoprTransportError::Api(format!(
                        "Failed to resolve {domain} to an IPv4 address. Does the DNS entry has an A record?"
                    )))?
                    .ip();

                out.push(ip.into())
            }
            multiaddr::Protocol::Dns6(domain) => {
                let ip = format!("{domain}:443") // dummy port, irrevelant at this point
                    .to_socket_addrs()
                    .map_err(|e| crate::errors::HoprTransportError::Api(e.to_string()))?
                    .filter(|sa| sa.is_ipv6())
                    .collect::<Vec<_>>()
                    .first()
                    .ok_or(crate::errors::HoprTransportError::Api(format!(
                        "Failed to resolve {domain} to an IPv6 address. Does the DNS entry has an AAAA record?"
                    )))?
                    .ip();

                out.push(ip.into())
            }
            _ => out.push(proto),
        }
    }

    Ok(out)
}

pub type TicketAggregationRequestType = OutboundRequestId;
pub type TicketAggregationResponseType = ResponseChannel<Result<Ticket, String>>;

pub struct HoprSwarm {
    pub(crate) swarm: libp2p::Swarm<HoprNetworkBehavior>,
}

impl HoprSwarm {
    pub async fn new(
        me: &OffchainKeypair,
        my_multiaddresses: Vec<multiaddr::Multiaddr>,
        protocol_cfg: ProtocolConfig,
    ) -> Self {
        let identity: libp2p::identity::Keypair = (me).into();

        let mut swarm = core_p2p::build_p2p_network(identity, protocol_cfg)
            .await
            .expect("swarm must be constructible");

        for multiaddress in my_multiaddresses.iter() {
            match resolve_dns_if_any(multiaddress) {
                Ok(ma) => {
                    if let Err(e) = swarm.listen_on(ma.clone()) {
                        error!("Failed to listen_on using the multiaddress '{multiaddress}': {e}");

                        match replace_transport_with_unspecified(&ma) {
                            Ok(ma) => {
                                if let Err(e) = swarm.listen_on(ma.clone()) {
                                    error!("Failed to listen_on also using the unspecified multiaddress '{ma}': {e}",);
                                } else {
                                    info!("Successfully started listening on {ma} (from {multiaddress})");
                                    swarm.add_external_address(multiaddress.clone());
                                }
                            }
                            Err(e) => {
                                error!("Failed to transform the multiaddress '{ma}' to unspecified: {e}")
                            }
                        }
                    } else {
                        info!("Successfully started listening on {ma} (from {multiaddress})");
                        swarm.add_external_address(multiaddress.clone());
                    }
                }
                Err(_) => error!("Failed to transform the multiaddress '{multiaddress}' - skipping"),
            }
        }

        // NOTE: This would be a valid check but is not immediate
        // assert!(
        //     swarm.listeners().count() > 0,
        //     "The node failed to listen on at least one of the specified interfaces"
        // );

        Self { swarm }
    }

    pub fn peer_id(&self) -> PeerId {
        *self.swarm.local_peer_id()
    }
}

impl From<HoprSwarm> for libp2p::Swarm<HoprNetworkBehavior> {
    fn from(value: HoprSwarm) -> Self {
        value.swarm
    }
}

pub struct SwarmEventLoop {
    network_update_input: futures::channel::mpsc::Receiver<NetworkTriggeredEvent>,
    indexer_update_input: async_channel::Receiver<PeerTransportEvent>,
    ack_interactions: AcknowledgementInteraction,
    pkt_interactions: PacketInteraction,
    ticket_aggregation_interactions:
        TicketAggregationInteraction<ResponseChannel<Result<Ticket, String>>, OutboundRequestId>,
    heartbeat_requests: api::HeartbeatRequester,
    heartbeat_responds: api::HeartbeatResponder,
    manual_ping_requests: api::ManualPingRequester,
    manual_ping_responds: api::HeartbeatResponder,
}

impl std::fmt::Debug for SwarmEventLoop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SwarmEventLoop").finish()
    }
}

impl SwarmEventLoop {
    #[allow(clippy::too_many_arguments)] // TODO: refactor this function into a reasonable group of components once fully rearchitected
    pub fn new(
        network_update_input: futures::channel::mpsc::Receiver<NetworkTriggeredEvent>,
        indexer_update_input: async_channel::Receiver<PeerTransportEvent>,
        ack_interactions: AcknowledgementInteraction,
        pkt_interactions: PacketInteraction,
        ticket_aggregation_interactions: TicketAggregationInteraction<
            TicketAggregationResponseType,
            TicketAggregationRequestType,
        >,
        heartbeat_requests: api::HeartbeatRequester,
        heartbeat_responds: api::HeartbeatResponder,
        manual_ping_requests: api::ManualPingRequester,
        manual_ping_responds: api::HeartbeatResponder,
    ) -> Self {
        Self {
            network_update_input,
            indexer_update_input,
            ack_interactions,
            pkt_interactions,
            ticket_aggregation_interactions,
            heartbeat_requests,
            heartbeat_responds,
            manual_ping_requests,
            manual_ping_responds,
        }
    }

    /// Main p2p loop that instantiates a new libp2p::Swarm instance and sets up listening and reacting pipelines
    /// running in a neverending loop future.
    ///
    /// The function represents the entirety of the business logic of the hopr daemon related to core operations.
    ///
    /// This future can only be resolved by an unrecoverable error or a panic.
    pub async fn run<T>(
        self,
        swarm: HoprSwarm,
        version: String,
        network: Arc<Network<T>>,
        on_transport_output: UnboundedSender<TransportOutput>,
        on_acknowledged_ticket: UnboundedSender<AcknowledgedTicket>,
    ) where
        T: HoprDbPeersOperations + Sync + Send + std::fmt::Debug + 'static,
    {
        let me_peer_id = swarm.peer_id();

        let mut swarm: libp2p::Swarm<HoprNetworkBehavior> = swarm.into();

        let mut heartbeat_responds = self.heartbeat_responds;
        let mut manual_ping_responds = self.manual_ping_responds;

        let mut ack_writer = self.ack_interactions.writer();
        let mut pkt_writer = self.pkt_interactions.writer();
        let mut aggregation_writer = self.ticket_aggregation_interactions.writer();

        let mut active_manual_pings: HashSet<libp2p::request_response::OutboundRequestId> = HashSet::new();
        let mut active_aggregation_requests: HashMap<
            libp2p::request_response::OutboundRequestId,
            TicketAggregationFinalizer,
        > = HashMap::new();

        let mut allowed_peers: HashSet<PeerId> = HashSet::new();

        let inputs = (
            self.heartbeat_requests.map(Inputs::Heartbeat),
            self.manual_ping_requests.map(Inputs::ManualPing),
            self.network_update_input.map(Inputs::NetworkUpdate),
            self.ack_interactions.map(Inputs::Acknowledgement),
            self.pkt_interactions.map(Inputs::Message),
            self.ticket_aggregation_interactions.map(Inputs::TicketAggregation),
            self.indexer_update_input.map(Inputs::Indexer),
        )
            .merge()
            .fuse();

        pin_mut!(inputs);

        loop {
            select! {
                input = inputs.select_next_some() => match input {
                    Inputs::Heartbeat(api::HeartbeatChallenge(peer, challenge)) => {
                        trace!("transport input - heartbeat - executing ping to peer '{peer}'");
                        swarm.behaviour_mut().heartbeat.send_request(&peer, Ping(challenge));
                    },
                    Inputs::ManualPing(api::ManualPingChallenge(peer, challenge)) => {
                        trace!("transport input - manual ping - executing ping to peer '{peer}'");
                        let req_id = swarm.behaviour_mut().heartbeat.send_request(&peer, Ping(challenge));
                        active_manual_pings.insert(req_id);
                    },
                    Inputs::NetworkUpdate(event) => match event {
                        NetworkTriggeredEvent::CloseConnection(peer) => {
                            debug!("transport input - network event - closing connection to '{peer}' (reason: low ping connection quality)");
                            if swarm.is_connected(&peer) {
                                let _ = swarm.disconnect_peer_id(peer);
                            }
                        },
                        NetworkTriggeredEvent::UpdateQuality(_, _) => {}
                    },
                    Inputs::Acknowledgement(task) => match task {
                        AckProcessed::Receive(peer, reply) => {
                            debug!("transport input - ack - received an acknowledgement from '{peer}'");
                            if let Ok(reply) = reply {
                                match reply {
                                    AckResult::Sender(half_key_challenge) => {
                                        if let Err(e) = on_transport_output.unbounded_send(TransportOutput::Sent(half_key_challenge)) {
                                            error!("transport input - ack - failed to emit received acknowledgement: {e}")
                                        }
                                    },
                                    AckResult::RelayerWinning(acknowledged_ticket) => {
                                        if let Err(e) = on_acknowledged_ticket.unbounded_send(acknowledged_ticket) {
                                            error!("transport input - ack -failed to emit acknowledged ticket: {e}");
                                        }
                                    }
                                    AckResult::RelayerLosing => {}
                                }
                            }
                        },
                        AckProcessed::Send(peer, ack) => {
                            trace!("transport input - ack - sending an acknowledgement to '{peer}'");
                            let _request_id = swarm.behaviour_mut().ack.send_request(&peer, ack);
                        }
                    }
                    Inputs::Message(task) => match task {
                        MsgProcessed::Receive(peer, data, ack) => {
                            debug!("transport input - msg - received packet from '{peer}'");
                            if let Err(e) = on_transport_output.unbounded_send(TransportOutput::Received(data)) {
                                error!("transport input - msg - failed to store a received message in the inbox: {}", e);
                            }

                            if let Err(e) = ack_writer.send_acknowledgement(peer, ack) {
                                error!("transport input - msg - failed to acknowledge the received final packet: {e}");
                            }
                        },
                        MsgProcessed::Send(peer, octets) => {
                            debug!("transport input - msg - sending packet as source to '{peer}'");
                            let _request_id = swarm.behaviour_mut().msg.send_request(&peer, octets);
                        },
                        MsgProcessed::Forward(peer, octets, previous_peer, ack) => {
                            debug!("transport input - msg - forwarding packet from '{previous_peer}' to '{peer}'");
                            let _request_id = swarm.behaviour_mut().msg.send_request(&peer, octets);
                            if let Err(e) = ack_writer.send_acknowledgement(previous_peer, ack) {
                                error!("transport input - msg - failed to acknowledge relayed packet: {e}");
                            }
                        }
                    },
                    Inputs::TicketAggregation(task) => match task {
                        TicketAggregationProcessed::Send(peer, acked_tickets, finalizer) => {
                            debug!("transport input - ticket aggregation - send request to '{peer}' to aggregate {} tickets", acked_tickets.len());
                            let request_id = swarm.behaviour_mut().ticket_aggregation.send_request(&peer, acked_tickets);
                            active_aggregation_requests.insert(request_id, finalizer);
                        },
                        TicketAggregationProcessed::Reply(peer, ticket, response) => {
                            debug!("transport input - ticket aggregation - responding to request by '{peer}'");
                            if swarm.behaviour_mut().ticket_aggregation.send_response(response, ticket).is_err() {
                                error!("transport input - ticket aggregation - failed to send response to '{peer}'");
                            }
                        },
                        TicketAggregationProcessed::Receive(peer, acked_ticket, request) => {
                            if let Err(e) = on_acknowledged_ticket.unbounded_send(acked_ticket) {
                                error!("transport input - ticket aggregation - failed to emit acknowledged aggregated ticket to '{peer}': {e}");
                            }

                            match active_aggregation_requests.remove(&request) {
                                Some(finalizer) => finalizer.finalize(),
                                None => {
                                    warn!("transport input - ticket aggregation - response already handled")
                                }
                            }
                        }
                    },
                    Inputs::Indexer(task) => match task {
                        PeerTransportEvent::Allow(peer) => {
                            debug!("transport input - indexer - allowing '{peer}'");
                            let _ = allowed_peers.insert(peer);
                        }
                        PeerTransportEvent::Ban(peer) => {
                            debug!("transport input - indexer - banning '{peer}'");
                            allowed_peers.remove(&peer);

                            if swarm.is_connected(&peer) {
                                match swarm.disconnect_peer_id(peer) {
                                    Ok(_) => debug!("Peer '{peer}' disconnected on network registry update"),
                                    Err(e) => error!("Failed to disconnect peer '{peer}' on network registry update: {:?}", e)
                                }
                            }
                        },
                        PeerTransportEvent::Announce(peer, multiaddresses) => {
                            if peer != me_peer_id {
                                trace!("transport input - indexer - processing announcement for '{peer}' with addresses: '{multiaddresses:?}'");
                                for multiaddress in multiaddresses.iter() {
                                    if !swarm.is_connected(&peer) {
                                        match swarm.dial(multiaddress.clone()) {
                                            Ok(_) => {
                                                debug!("transport input - indexer - storing '{multiaddress}' as valid for '{peer}'");
                                                swarm.behaviour_mut().heartbeat.add_address(&peer, multiaddress.clone());
                                                swarm.behaviour_mut().msg.add_address(&peer, multiaddress.clone());
                                                swarm.behaviour_mut().ack.add_address(&peer, multiaddress.clone());
                                                swarm.behaviour_mut().ticket_aggregation.add_address(&peer, multiaddress.clone());
                                            },
                                            Err(e) => {
                                                warn!("transport input - indexer - failed to dial an announced peer '{peer}': {e}, ignoring the address '{multiaddress}'");
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
                event = swarm.select_next_some() => match event {
                    // ---------------
                    // msg protocol
                    SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::Message(libp2p::request_response::Event::<Box<[u8]>, ()>::Message {
                        peer,
                        message:
                        libp2p::request_response::Message::<Box<[u8]>, ()>::Request {
                            request_id, request, channel
                        },
                    })) => {
                        debug!("transport protocol - p2p - msg - received a message from {peer}");

                        if let Err(e) = pkt_writer.receive_packet(request, peer) {
                            error!("transport protocol - p2p - msg - failed to process a message from '{peer}': {e} (#{request_id})");
                        };

                        if swarm.behaviour_mut().msg.send_response(channel, ()).is_err() {
                            error!("transport protocol - p2p - msg - failed to send a response to '{peer}', likely a timeout");
                        };
                    },
                    SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::Message(libp2p::request_response::Event::<Box<[u8]>, ()>::Message {
                        peer,
                        message:
                        libp2p::request_response::Message::<Box<[u8]>, ()>::Response {
                            request_id, ..
                        },
                    })) => {
                        trace!("transport protocol - p2p - msg - received a response for sending message with id {request_id} from '{peer}'");
                    },
                    SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::Message(libp2p::request_response::Event::<Box<[u8]>, ()>::OutboundFailure {
                        peer, error, request_id
                    })) => {
                        error!("transport protocol - p2p - msg - failed to send a message (#{}) to peer {} with an error: {}", request_id, peer, error);
                    },
                    SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::Message(libp2p::request_response::Event::<Box<[u8]>, ()>::InboundFailure {..}))
                    | SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::Message(libp2p::request_response::Event::<Box<[u8]>, ()>::ResponseSent {..})) => {
                    },
                    // ---------------
                    // ack protocol
                    SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::Acknowledgement(libp2p::request_response::Event::<Acknowledgement,()>::Message {
                        peer,
                        message:
                        libp2p::request_response::Message::<Acknowledgement,()>::Request {
                            request_id, request, channel
                        },
                    })) => {
                        debug!("transport protocol - p2p - ack - received an acknowledgment from '{peer}'");

                        if let Err(e) = ack_writer.receive_acknowledgement(peer, request) {
                            error!("transport protocol - p2p - ack - failed to process an acknowledgement from '{peer}': {e} (#{request_id})");
                        };

                        if swarm.behaviour_mut().ack.send_response(channel, ()).is_err() {
                            error!("transport protocol - p2p - ack - failed to send a response to '{peer}', likely a timeout");
                        };
                    },
                    SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::Acknowledgement(libp2p::request_response::Event::<Acknowledgement,()>::Message {
                        peer,
                        message:
                        libp2p::request_response::Message::<Acknowledgement,()>::Response {
                            request_id, ..
                        },
                    })) => {
                        trace!("transport protocol - p2p - ack - received a response for sending message with id {} from {}", request_id, peer);
                    },
                    SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::Acknowledgement(libp2p::request_response::Event::<Acknowledgement,()>::OutboundFailure {
                        peer, error, ..
                    })) => {
                        error!("transport protocol - p2p - ack - failed to send an acknowledgement to '{peer}': {error}");
                    },
                    SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::Acknowledgement(libp2p::request_response::Event::<Acknowledgement,()>::InboundFailure {..}))
                    | SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::Acknowledgement(libp2p::request_response::Event::<Acknowledgement,()>::ResponseSent {..})) => {
                        // debug!("Discarded messages not relevant for the protocol!");
                    },
                    // --------------
                    // ticket aggregation protocol
                    SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::TicketAggregation(libp2p::request_response::Event::<Vec<legacy::AcknowledgedTicket>, std::result::Result<Ticket,String>>::Message {
                        peer,
                        message:
                        libp2p::request_response::Message::<Vec<legacy::AcknowledgedTicket>, std::result::Result<Ticket,String>>::Request {
                            request_id, request, channel
                        },
                    })) => {
                        debug!("transport protocol - p2p - ticket aggregation - received an aggregation request {request_id} from '{peer}'");
                        let request = request.into_iter().map(TransferableWinningTicket::from).collect::<Vec<_>>();
                        if let Err(e) = aggregation_writer.receive_aggregation_request(peer, request, channel) {
                            error!("transport protocol - p2p - ticket aggregation - failed to aggregate tickets for '{peer}': {e}");
                        }
                    },
                    SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::TicketAggregation(libp2p::request_response::Event::<Vec<legacy::AcknowledgedTicket>, std::result::Result<Ticket,String>>::Message {
                        peer,
                        message:
                        libp2p::request_response::Message::<Vec<legacy::AcknowledgedTicket>, std::result::Result<Ticket,String>>::Response {
                            request_id, response
                        },
                    })) => {
                        if let Err(e) = aggregation_writer.receive_ticket(peer, response, request_id) {
                            error!("transport protocol - p2p - ticket aggregation - error while handling aggregated ticket from '{peer}': {e}");
                        }
                    },
                    SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::TicketAggregation(libp2p::request_response::Event::<Vec<legacy::AcknowledgedTicket>, std::result::Result<Ticket,String>>::OutboundFailure {
                        peer, request_id, error,
                    })) => {
                        error!("transport protocol - p2p - ticket aggregation - failed to send an aggergation request #{request_id} to {peer}: {error}");
                    },
                    SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::TicketAggregation(libp2p::request_response::Event::<Vec<legacy::AcknowledgedTicket>, std::result::Result<Ticket,String>>::InboundFailure {
                        peer, request_id, error})) => {
                        warn!("transport protocol - p2p - ticket aggregation - encountered inbound failure for '{peer}' (#{request_id}): {error}")
                    }
                    SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::TicketAggregation(libp2p::request_response::Event::<Vec<legacy::AcknowledgedTicket>, std::result::Result<Ticket,String>>::ResponseSent {..})) => {
                    },
                    // --------------
                    // heartbeat protocol
                    SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::Heartbeat(libp2p::request_response::Event::<Ping,Pong>::Message {
                        peer,
                        message:
                        libp2p::request_response::Message::<Ping,Pong>::Request {
                            request_id, request, channel
                        },
                    })) => {
                        trace!("transport protocol - p2p - heartbeat - received a Ping request from '{peer}' (#{request_id})");
                        let challenge_response = api::HeartbeatResponder::generate_challenge_response(&request.0);
                        if swarm.behaviour_mut().heartbeat.send_response(channel, Pong(challenge_response, version.clone())).is_err() {
                            error!("transport protocol - p2p - heartbeat - failed to reply to ping request");
                        };
                    },
                    SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::Heartbeat(libp2p::request_response::Event::<Ping,Pong>::Message {
                        peer,
                        message:
                        libp2p::request_response::Message::<Ping,Pong>::Response {
                            request_id, response
                        },
                    })) => {
                        if active_manual_pings.take(&request_id).is_some() {
                            trace!("transport protocol - p2p - heartbeat - processing manual ping response from '{peer}'");
                            if let Err(e) = manual_ping_responds.record_pong((peer, Ok((response.0, response.1)))) {
                                error!("transport protocol - p2p - heartbeat - failed to record manual ping response: {e}");
                            }
                        } else {
                            trace!("transport protocol - p2p - heartbeat - processing heartbeat ping response from peer '{peer}'");
                            if let Err(e) = heartbeat_responds.record_pong((peer, Ok((response.0, response.1)))) {
                                error!("transport protocol - p2p - heartbeat - failed to record heartbeat response: {e}");
                            }
                        }
                    },
                    SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::Heartbeat(libp2p::request_response::Event::<Ping,Pong>::OutboundFailure {
                        peer, ..    //request_id, error,
                    })) => {
                        if let Err(e) = heartbeat_responds.record_pong((peer, Err(()))) {
                            error!("transport protocol - p2p - failed to update heartbeat mechanism with response: {e}");
                        }
                    },
                    SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::Heartbeat(libp2p::request_response::Event::<Ping,Pong>::InboundFailure {
                        peer, error, ..     // request_id
                    })) => {
                        warn!("transport protocol - p2p - heartbeat - inbound failure for peer '{peer}': {error}")
                    }
                    SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::Heartbeat(libp2p::request_response::Event::<Ping,Pong>::ResponseSent {..})) => {},
                    SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::KeepAlive(_)) => {}
                    SwarmEvent::ConnectionEstablished {
                        peer_id,
                        connection_id,
                        ..
                        // endpoint,
                        // num_established,
                        // concurrent_dial_errors,
                        // established_in,
                    } => {
                        debug!("transport - p2p - connection ({connection_id}) established with {peer_id}");

                        #[cfg(all(feature = "prometheus", not(test)))]
                        {
                            METRIC_TRANSPORT_P2P_OPEN_CONNECTION_COUNT.increment(1.0);
                        }

                        if !allowed_peers.contains(&peer_id) {
                            info!("transport - p2p - DISCONNECTING '{peer_id}': not allowed in the network registry)");
                            let _ = swarm.disconnect_peer_id(peer_id);
                        } else {
                            let network = network.clone();
                            let _ = spawn(async move {
                                if !network.has(&peer_id).await {
                                    if let Err(e) = network.add(&peer_id, PeerOrigin::IncomingConnection, vec![]).await {
                                        error!("transport - p2p - failed to update the record for '{peer_id}': {e}")
                                    }
                                }
                            }).await;
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
                        debug!("transport - p2p - connection closed for peer '{peer_id}': {cause:?}");

                        #[cfg(all(feature = "prometheus", not(test)))]
                        {
                            METRIC_TRANSPORT_P2P_OPEN_CONNECTION_COUNT.decrement(1.0);
                        }
                    },
                    SwarmEvent::IncomingConnection {
                        connection_id,
                        local_addr,
                        send_back_addr,
                    } => {
                        debug!("transport - p2p - incoming connection at {local_addr} from {send_back_addr} ({connection_id:?})");
                    },
                    SwarmEvent::IncomingConnectionError {
                        local_addr,
                        error,
                        ..
                        // connection_id,
                        // send_back_addr,
                    } => {
                        error!("transport - p2p - incoming connection error on {local_addr}: {error}")
                    },
                    SwarmEvent::OutgoingConnectionError {
                        connection_id,
                        error,
                        peer_id
                    } => {
                        error!("transport - p2p - outgoing connection error for peer '{peer_id:?}' ({connection_id:?}): {error}")
                    },
                    SwarmEvent::NewListenAddr {
                        listener_id,
                        ..
                        // address,
                    } => {
                        debug!("transport - p2p - new listen address on {listener_id:?}")
                    },
                    SwarmEvent::ExpiredListenAddr {
                        listener_id,
                        ..
                        // address,
                    } => {
                        debug!("transport - p2p - expired listen address on {listener_id:?}")
                    },
                    SwarmEvent::ListenerClosed {
                        listener_id,
                        ..
                        // addresses,
                        // reason,
                    } => {
                        debug!("transport - p2p - listener closed on {listener_id:?}", )
                    },
                    SwarmEvent::ListenerError {
                        listener_id,
                        error,
                    } => {
                        debug!("transport - p2p - listener error for {listener_id:?}: {error}")
                    },
                    SwarmEvent::Dialing {
                        peer_id,
                        connection_id,
                    } => {
                        if let Some(peer_id) = peer_id {
                            if !allowed_peers.contains(&peer_id) {
                                info!("transport - p2p - dialing '{peer_id}': not allowed in the network registry)");
                                let _ = swarm.disconnect_peer_id(peer_id);
                            } else {
                                debug!("transport - p2p - dialing peer {peer_id:?} ({connection_id:?}")
                            }
                        }
                    },
                    _ => error!("transport - p2p - unimplemented message type in p2p processing chain encountered")
                }
            }
        }
    }
}
