use futures::{channel::mpsc::UnboundedSender, pin_mut, select, StreamExt};
use futures_concurrency::stream::Merge;
use libp2p::{request_response::OutboundRequestId, PeerId};
use std::{collections::HashMap, num::NonZeroU8};
use tracing::{debug, error, info, trace, warn};

use core_network::{messaging::ControlMessage, network::NetworkTriggeredEvent, ping::PingQueryReplier};
use core_protocol::{
    ack::processor::{AckProcessed, AckResult, AcknowledgementInteraction},
    config::ProtocolConfig,
    msg::processor::{MsgProcessed, PacketInteraction},
    ticket_aggregation::processor::{
        TicketAggregationFinalizer, TicketAggregationInteraction, TicketAggregationProcessed,
    },
};
use hopr_internal_types::prelude::*;

use crate::{
    constants,
    errors::Result,
    libp2p::{request_response::ResponseChannel, swarm::SwarmEvent},
    multiaddrs::{replace_transport_with_unspecified, resolve_dns_if_any, Multiaddr},
    HoprNetworkBehavior, HoprNetworkBehaviorEvent, PeerDiscovery, Ping, Pong, TransportOutput,
};

#[cfg(all(feature = "prometheus", not(test)))]
use hopr_metrics::metrics::SimpleGauge;

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_TRANSPORT_P2P_OPEN_CONNECTION_COUNT: SimpleGauge = SimpleGauge::new(
        "hopr_transport_p2p_opened_connection_count",
        "Number of currently open connections"
    ).unwrap();
}

/// Build objects comprising the p2p network.
///
/// Returns a built [libp2p::Swarm] object implementing the HoprNetworkBehavior functionality.
async fn build_p2p_network(
    me: libp2p::identity::Keypair,
    network_update_input: futures::channel::mpsc::Receiver<NetworkTriggeredEvent>,
    indexer_update_input: futures::channel::mpsc::UnboundedReceiver<PeerDiscovery>,
    protocol_cfg: ProtocolConfig,
) -> Result<libp2p::Swarm<HoprNetworkBehavior>> {
    let tcp_upgrade = libp2p::core::upgrade::SelectUpgrade::new(
        libp2p::yamux::Config::default(),
        libp2p_mplex::MplexConfig::new()
            .set_max_num_streams(1024)
            .set_max_buffer_size(32)
            .set_split_send_size(8 * 1024)
            .set_max_buffer_behaviour(libp2p_mplex::MaxBufferBehaviour::Block)
            .clone(),
    );

    let me_peerid: PeerId = me.public().into();

    #[cfg(feature = "runtime-async-std")]
    let swarm = libp2p::SwarmBuilder::with_existing_identity(me)
        .with_async_std()
        .with_tcp(Default::default(), libp2p::noise::Config::new, move || tcp_upgrade)
        .map_err(|e| crate::errors::P2PError::Libp2p(e.to_string()))?
        .with_quic()
        .with_dns()
        .await;

    // Both features could be enabled during testing, therefore we only use tokio when its
    // exclusively enabled.
    #[cfg(all(feature = "runtime-tokio", not(feature = "runtime-async-std")))]
    let swarm = libp2p::SwarmBuilder::with_existing_identity(me)
        .with_tokio()
        .with_tcp(Default::default(), libp2p::noise::Config::new, || tcp_upgrade)
        .map_err(|e| crate::errors::P2PError::Libp2p(e.to_string()))?
        .with_quic()
        .with_dns();

    Ok(swarm
        .map_err(|e| crate::errors::P2PError::Libp2p(e.to_string()))?
        .with_behaviour(|_key| {
            HoprNetworkBehavior::new(
                me_peerid,
                network_update_input,
                indexer_update_input,
                protocol_cfg.msg,
                protocol_cfg.ack,
                protocol_cfg.heartbeat,
                protocol_cfg.ticket_aggregation,
            )
        })
        .map_err(|e| crate::errors::P2PError::Libp2p(e.to_string()))?
        .with_swarm_config(|cfg| {
            cfg.with_dial_concurrency_factor(
                NonZeroU8::new(
                    std::env::var("HOPR_INTERNAL_LIBP2P_MAX_CONCURRENTLY_DIALED_PEER_COUNT")
                        .map(|v| v.trim().parse::<u8>().unwrap_or(u8::MAX))
                        .unwrap_or(constants::HOPR_SWARM_CONCURRENTLY_DIALED_PEER_COUNT),
                )
                .expect("concurrently dialed peer count must be > 0"),
            )
            .with_max_negotiating_inbound_streams(
                std::env::var("HOPR_INTERNAL_LIBP2P_MAX_NEGOTIATING_INBOUND_STREAM_COUNT")
                    .map(|v| v.trim().parse::<usize>().unwrap_or(128))
                    .unwrap_or(constants::HOPR_SWARM_CONCURRENTLY_NEGOTIATING_INBOUND_PEER_COUNT),
            )
            .with_idle_connection_timeout(constants::HOPR_SWARM_IDLE_CONNECTION_TIMEOUT)
        })
        .build())
}

pub struct HoprSwarm {
    pub(crate) swarm: libp2p::Swarm<HoprNetworkBehavior>,
}

impl HoprSwarm {
    pub async fn new(
        identity: libp2p::identity::Keypair,
        network_update_input: futures::channel::mpsc::Receiver<NetworkTriggeredEvent>,
        indexer_update_input: futures::channel::mpsc::UnboundedReceiver<PeerDiscovery>,
        my_multiaddresses: Vec<Multiaddr>,
        protocol_cfg: ProtocolConfig,
    ) -> Self {
        let mut swarm = build_p2p_network(identity, network_update_input, indexer_update_input, protocol_cfg)
            .await
            .expect("swarm must be constructible");

        for multiaddress in my_multiaddresses.iter() {
            match resolve_dns_if_any(multiaddress) {
                Ok(ma) => {
                    if let Err(e) = swarm.listen_on(ma.clone()) {
                        error!("Failed to listen_on using '{multiaddress}': {e}");

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

    pub fn with_processors(
        self,
        ack_interactions: AcknowledgementInteraction,
        pkt_interactions: PacketInteraction,
        ticket_aggregation_interactions: TicketAggregationInteraction<
            TicketAggregationResponseType,
            TicketAggregationRequestType,
        >,
        heartbeat_requests: futures::channel::mpsc::UnboundedReceiver<(PeerId, PingQueryReplier)>,
    ) -> HoprSwarmWithProcessors {
        HoprSwarmWithProcessors {
            swarm: self,
            ack_interactions,
            pkt_interactions,
            ticket_aggregation_interactions,
            heartbeat_requests,
        }
    }
}

impl From<HoprSwarm> for libp2p::Swarm<HoprNetworkBehavior> {
    fn from(value: HoprSwarm) -> Self {
        value.swarm
    }
}

/// Composition of all inputs allowing to produce a single stream of
/// input events passed into the swarm processing logic.
#[derive(Debug)]
pub enum Inputs {
    Heartbeat((PeerId, PingQueryReplier)),
    Message(MsgProcessed),
    TicketAggregation(
        TicketAggregationProcessed<ResponseChannel<std::result::Result<Ticket, String>>, OutboundRequestId>,
    ),
    Acknowledgement(AckProcessed),
}

impl From<(PeerId, PingQueryReplier)> for Inputs {
    fn from(value: (PeerId, PingQueryReplier)) -> Self {
        Self::Heartbeat(value)
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

impl From<TicketAggregationProcessed<TicketAggregationResponseType, TicketAggregationRequestType>> for Inputs {
    fn from(value: TicketAggregationProcessed<TicketAggregationResponseType, TicketAggregationRequestType>) -> Self {
        Self::TicketAggregation(value)
    }
}

use hopr_internal_types::legacy;

pub type TicketAggregationRequestType = OutboundRequestId;
pub type TicketAggregationResponseType = ResponseChannel<std::result::Result<Ticket, String>>;

pub struct HoprSwarmWithProcessors {
    swarm: HoprSwarm,
    ack_interactions: AcknowledgementInteraction,
    pkt_interactions: PacketInteraction,
    ticket_aggregation_interactions:
        TicketAggregationInteraction<TicketAggregationResponseType, TicketAggregationRequestType>,
    heartbeat_requests: futures::channel::mpsc::UnboundedReceiver<(PeerId, PingQueryReplier)>,
}

impl std::fmt::Debug for HoprSwarmWithProcessors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SwarmEventLoop").finish()
    }
}

impl HoprSwarmWithProcessors {
    /// Main p2p loop that instantiates a new libp2p::Swarm instance and sets up listening and reacting pipelines
    /// running in a neverending loop future.
    ///
    /// The function represents the entirety of the business logic of the hopr daemon related to core operations.
    ///
    /// This future can only be resolved by an unrecoverable error or a panic.
    pub async fn run(
        self,
        version: String,
        on_transport_output: UnboundedSender<TransportOutput>,
        on_acknowledged_ticket: UnboundedSender<AcknowledgedTicket>,
    ) {
        let mut swarm: libp2p::Swarm<HoprNetworkBehavior> = self.swarm.into();

        let mut ack_writer = self.ack_interactions.writer();
        let mut pkt_writer = self.pkt_interactions.writer();
        let mut aggregation_writer = self.ticket_aggregation_interactions.writer();

        // NOTE: an improvement would be a forgetting cache for the active requests
        let mut active_pings: HashMap<libp2p::request_response::OutboundRequestId, PingQueryReplier> = HashMap::new();
        let mut active_aggregation_requests: HashMap<
            libp2p::request_response::OutboundRequestId,
            TicketAggregationFinalizer,
        > = HashMap::new();

        let inputs = (
            self.heartbeat_requests.map(Inputs::Heartbeat),
            self.ack_interactions.map(Inputs::Acknowledgement),
            self.pkt_interactions.map(Inputs::Message),
            self.ticket_aggregation_interactions.map(Inputs::TicketAggregation),
        )
            .merge()
            .fuse();

        pin_mut!(inputs);

        loop {
            select! {
                input = inputs.select_next_some() => match input {
                    Inputs::Heartbeat((peer, replier)) => {
                        trace!("transport input - heartbeat - executing ping to peer '{peer}'");
                        let req_id = swarm.behaviour_mut().heartbeat.send_request(&peer, Ping(replier.challenge()));
                        active_pings.insert(req_id, replier);
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
                            let _req_id = swarm.behaviour_mut().ack.send_request(&peer, ack);
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
                            let req_id = swarm.behaviour_mut().ticket_aggregation.send_request(&peer, acked_tickets);
                            active_aggregation_requests.insert(req_id, finalizer);
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
                },
                event = swarm.select_next_some() => match event {
                    // ---------------
                    // msg/ack protocol
                    SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::MessageWithAcknowledgement(libp2p::request_response::Event::<Box<[u8]>, Acknowledgement>::Message {
                        peer,
                        message:
                        libp2p::request_response::Message::<Box<[u8]>, Acknowledgement>::Request {
                            request_id, request, .. //channel
                        },
                    })) => {
                        debug!("transport protocol - p2p - msg/ack 0.1.0 - received a message from {peer}");

                        if let Err(e) = pkt_writer.receive_packet(request, peer) {
                            error!("transport protocol - p2p - msg/ack 0.1.0 - failed to process a message from '{peer}': {e} (#{request_id})");
                        };

                        // TODO: Add implementation for sending a response to the sender

                        // if swarm.behaviour_mut().msg.send_response(channel, ()).is_err() {
                        //     error!("transport protocol - p2p - msg - failed to send a response to '{peer}', likely a timeout");
                        // };
                    },
                    SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::MessageWithAcknowledgement(libp2p::request_response::Event::<Box<[u8]>, Acknowledgement>::Message {
                        peer,
                        message:
                        libp2p::request_response::Message::<Box<[u8]>, Acknowledgement>::Response {
                            request_id, ..
                        },
                    })) => {
                        trace!("transport protocol - p2p - msg/ack 0.1.0 - received a response for sending message with id {request_id} from '{peer}'");
                    },
                    SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::MessageWithAcknowledgement(libp2p::request_response::Event::<Box<[u8]>, Acknowledgement>::OutboundFailure {
                        peer, error, request_id
                    })) => {
                        error!("transport protocol - p2p - msg/ack 0.1.0 - failed to send a message (#{request_id}) to '{peer}': {error}");
                    },
                    SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::MessageWithAcknowledgement(libp2p::request_response::Event::<Box<[u8]>, Acknowledgement>::InboundFailure {..}))
                    | SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::MessageWithAcknowledgement(libp2p::request_response::Event::<Box<[u8]>, Acknowledgement>::ResponseSent {..})) => {
                    },
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
                        if let Ok(challenge_response) = ControlMessage::generate_pong_response(&request.0)
                        {
                            if swarm.behaviour_mut().heartbeat.send_response(channel, Pong(challenge_response, version.clone())).is_err() {
                                error!("transport protocol - p2p - heartbeat - failed to reply to ping request");
                            };
                        }
                    },
                    SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::Heartbeat(libp2p::request_response::Event::<Ping,Pong>::Message {
                        peer,
                        message:
                        libp2p::request_response::Message::<Ping,Pong>::Response {
                            request_id, response
                        },
                    })) => {
                        if let Some(replier) = active_pings.remove(&request_id) {
                            trace!("transport protocol - p2p - heartbeat - processing manual ping response from '{peer}'");
                            replier.notify(response.0, response.1)
                        } else {
                            debug!("transport protocol - p2p - heartbeat - failed to find heartbeat replier for '{peer}'");
                        }
                    },
                    SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::Heartbeat(libp2p::request_response::Event::<Ping,Pong>::OutboundFailure {
                        request_id, peer, error,
                    })) => {
                        trace!("transport protocol - p2p - heartbeat - encountered outbound failure for '{peer}': {error}");
                        active_pings.remove(&request_id);
                    },
                    SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::Heartbeat(libp2p::request_response::Event::<Ping,Pong>::InboundFailure {
                        peer, error, ..     // request_id
                    })) => {
                        warn!("transport protocol - p2p - heartbeat - inbound failure for peer '{peer}': {error}")
                    }
                    SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::Heartbeat(libp2p::request_response::Event::<Ping,Pong>::ResponseSent {..})) => {},
                    SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::KeepAlive(_)) => {}
                    SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::Discovery(event)) => {
                        trace!(event = tracing::field::debug(&event), "transport - p2p - discovery - Received a discovery event");
                        match event {
                            crate::discovery::Event::NewPeerMultiddress(peer, multiaddress) => {
                                info!(peer = %peer, multiaddress = %multiaddress, "transport - p2p - discovery - New record");
                                // swarm.behaviour_mut().msg_ack.add_address(&peer, multiaddress.clone());
                                swarm.behaviour_mut().heartbeat.add_address(&peer, multiaddress.clone());
                                swarm.behaviour_mut().msg.add_address(&peer, multiaddress.clone());
                                swarm.behaviour_mut().ack.add_address(&peer, multiaddress.clone());
                                swarm.behaviour_mut().ticket_aggregation.add_address(&peer, multiaddress.clone());

                                if let Err(e) = swarm.dial(multiaddress.clone()) {
                                    error!(peer = %peer, address = %multiaddress,  "transport - p2p - discovery - Failed to dial peer: {e}");
                                }
                            },
                        }
                    },
                    SwarmEvent::ConnectionEstablished {
                        peer_id,
                        connection_id,
                        ..
                        // endpoint,
                        // num_established,
                        // concurrent_dial_errors,
                        // established_in,
                    } => {
                        debug!(peer = %peer_id, connection_id = %connection_id, "transport - p2p - connection established");

                        #[cfg(all(feature = "prometheus", not(test)))]
                        {
                            METRIC_TRANSPORT_P2P_OPEN_CONNECTION_COUNT.increment(1.0);
                        }
                    },
                    SwarmEvent::ConnectionClosed {
                        peer_id,
                        connection_id,
                        cause,
                        ..
                        // endpoint,
                        // num_established,
                    } => {
                        debug!(peer = %peer_id, connection_id = %connection_id, "transport - p2p - connection closed: {cause:?}");

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
                        trace!(local_addr = %local_addr, send_back_addr = %send_back_addr, connection_id = %connection_id, "transport - p2p - incoming connection");
                    },
                    SwarmEvent::IncomingConnectionError {
                        local_addr,
                        connection_id,
                        error,
                        send_back_addr,
                    } => {
                        error!(local_addr = %local_addr, send_back_addr = %send_back_addr, connection_id = %connection_id, "transport - p2p - incoming connection error: {error}")
                    },
                    SwarmEvent::OutgoingConnectionError {
                        connection_id,
                        error,
                        peer_id
                    } => {
                        error!(peer = tracing::field::debug(peer_id), connection_id = %connection_id, "transport - p2p - outgoing connection error: {error}")
                    },
                    SwarmEvent::NewListenAddr {
                        listener_id,
                        address,
                    } => {
                        debug!(listener_id = %listener_id, address = %address, "transport - p2p - new listen address")
                    },
                    SwarmEvent::ExpiredListenAddr {
                        listener_id,
                        address,
                    } => {
                        debug!(listener_id = %listener_id, address = %address, "transport - p2p - expired listen address")
                    },
                    SwarmEvent::ListenerClosed {
                        listener_id,
                        addresses,
                        reason,
                    } => {
                        debug!(listener_id = %listener_id, addresses = tracing::field::debug(addresses), "transport - p2p - listener closed: {reason:?}", )
                    },
                    SwarmEvent::ListenerError {
                        listener_id,
                        error,
                    } => {
                        debug!(listener_id = %listener_id, "transport - p2p - listener error: {error}")
                    },
                    SwarmEvent::Dialing {
                        peer_id,
                        connection_id,
                    } => {
                        debug!(peer = tracing::field::debug(peer_id), connection_id = %connection_id, "transport - p2p - dialing")
                    },
                    _ => error!("transport - p2p - unimplemented message type in p2p processing chain encountered")
                }
            }
        }
    }
}
