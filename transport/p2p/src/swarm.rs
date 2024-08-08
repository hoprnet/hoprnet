use futures::{channel::mpsc::UnboundedSender, pin_mut, select, StreamExt};
use futures_concurrency::stream::Merge;
use libp2p::{request_response::OutboundRequestId, PeerId};
use std::{collections::HashMap, num::NonZeroU8};
use tracing::{debug, error, info, trace, warn};

use core_network::{messaging::ControlMessage, network::NetworkTriggeredEvent, ping::PingQueryReplier};
use hopr_internal_types::prelude::*;
use hopr_transport_protocol::{
    config::ProtocolConfig,
    ticket_aggregation::processor::{
        TicketAggregationActions, TicketAggregationFinalizer, TicketAggregationInteraction, TicketAggregationProcessed,
    },
};

use crate::{
    constants,
    errors::Result,
    libp2p::{request_response::ResponseChannel, swarm::SwarmEvent},
    multiaddrs::{replace_transport_with_unspecified, resolve_dns_if_any, Multiaddr},
    HoprNetworkBehavior, HoprNetworkBehaviorEvent, PeerDiscovery, Ping, Pong,
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
    heartbeat_requests: futures::channel::mpsc::UnboundedReceiver<(PeerId, PingQueryReplier)>,
    ticket_aggregation_interactions: TicketAggregationInteraction<
        TicketAggregationResponseType,
        TicketAggregationRequestType,
    >,
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
                heartbeat_requests,
                ticket_aggregation_interactions,
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
        heartbeat_requests: futures::channel::mpsc::UnboundedReceiver<(PeerId, PingQueryReplier)>,
        ticket_aggregation_interactions: TicketAggregationInteraction<
            TicketAggregationResponseType,
            TicketAggregationRequestType,
        >,
        my_multiaddresses: Vec<Multiaddr>,
        protocol_cfg: ProtocolConfig,
    ) -> Self {
        let mut swarm = build_p2p_network(
            identity,
            network_update_input,
            indexer_update_input,
            heartbeat_requests,
            ticket_aggregation_interactions,
            protocol_cfg,
        )
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

    // TODO: rename to with_outputs
    pub fn with_processors(
        self,
        ack_to_send: futures::channel::mpsc::UnboundedReceiver<(PeerId, Acknowledgement)>,
        ack_received: futures::channel::mpsc::UnboundedSender<(PeerId, Acknowledgement)>,
        msg_to_send: futures::channel::mpsc::UnboundedReceiver<(PeerId, Box<[u8]>)>,
        msg_received: futures::channel::mpsc::UnboundedSender<(PeerId, Box<[u8]>)>,
        ticket_aggregation_writer: TicketAggregationActions<
            TicketAggregationResponseType,
            TicketAggregationRequestType,
        >,
    ) -> HoprSwarmWithProcessors {
        HoprSwarmWithProcessors {
            swarm: self,
            ack_to_send,
            ack_received,
            msg_to_send,
            msg_received,
            ticket_aggregation_writer,
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
    Message((PeerId, Box<[u8]>)),
    Acknowledgement((PeerId, Acknowledgement)),
}

impl From<(PeerId, Acknowledgement)> for Inputs {
    fn from(value: (PeerId, Acknowledgement)) -> Self {
        Self::Acknowledgement(value)
    }
}

impl From<(PeerId, Box<[u8]>)> for Inputs {
    fn from(value: (PeerId, Box<[u8]>)) -> Self {
        Self::Message(value)
    }
}

use hopr_internal_types::legacy;

pub type TicketAggregationRequestType = OutboundRequestId;
pub type TicketAggregationResponseType = ResponseChannel<std::result::Result<Ticket, String>>;

pub struct HoprSwarmWithProcessors {
    swarm: HoprSwarm,
    ack_to_send: futures::channel::mpsc::UnboundedReceiver<(PeerId, Acknowledgement)>,
    ack_received: futures::channel::mpsc::UnboundedSender<(PeerId, Acknowledgement)>,
    msg_to_send: futures::channel::mpsc::UnboundedReceiver<(PeerId, Box<[u8]>)>,
    msg_received: futures::channel::mpsc::UnboundedSender<(PeerId, Box<[u8]>)>,
    ticket_aggregation_writer: TicketAggregationActions<TicketAggregationResponseType, TicketAggregationRequestType>,
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
    pub async fn run(self, version: String, on_ack_received: UnboundedSender<AcknowledgedTicket>) {
        let mut swarm: libp2p::Swarm<HoprNetworkBehavior> = self.swarm.into();

        // NOTE: an improvement would be a forgetting cache for the active requests
        let mut active_pings: HashMap<libp2p::request_response::OutboundRequestId, PingQueryReplier> = HashMap::new();
        let mut active_aggregation_requests: HashMap<
            libp2p::request_response::OutboundRequestId,
            TicketAggregationFinalizer,
        > = HashMap::new();

        let mut aggregation_writer = self.ticket_aggregation_writer.clone();

        let inputs = (
            self.ack_to_send.map(Inputs::Acknowledgement),
            self.msg_to_send.map(Inputs::Message),
        )
            .merge()
            .fuse();

        pin_mut!(inputs);

        loop {
            select! {
                input = inputs.select_next_some() => match input {
                    Inputs::Acknowledgement((peer, ack)) => {
                        let req_id = swarm.behaviour_mut().ack.send_request(&peer, ack);
                        trace!(peer = %peer, request_id = %req_id, "transport - ack - Sending an acknowledgement");
                    },
                    Inputs::Message((peer, octets)) => {
                        let req_id = swarm.behaviour_mut().msg.send_request(&peer, octets);
                        trace!(peer = %peer, request_id = %req_id, "transport - msg - Sending a message");
                    },
                },
                event = swarm.select_next_some() => match event {
                    SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::Message(event)) => {
                        let _debug_span = tracing::span!(tracing::Level::DEBUG, "swarm protocol ACK", version = "0.1.0");
                            match event {
                            libp2p::request_response::Event::<Box<[u8]>, ()>::Message {
                                peer,
                                message,
                            } => match message {
                                libp2p::request_response::Message::<Box<[u8]>, ()>::Request {
                                    request_id, request, channel
                                } => {
                                    trace!(peer = %peer, request_id = %request_id, "Received a message");

                                    if let Err(e) = self.msg_received.unbounded_send((peer, request)) {
                                        error!(peer = %peer, request_id = %request_id, "Failed to process a message: {e}");
                                    };

                                    if swarm.behaviour_mut().msg.send_response(channel, ()).is_err() {
                                        error!("transport protocol - p2p - msg - failed to send a response to '{peer}'");
                                    };
                                },
                                libp2p::request_response::Message::<Box<[u8]>, ()>::Response {
                                    request_id, ..
                                } => {
                                    error!(peer = %peer, request_id = %request_id, "Failed to confirm receiving a message, likely a timeout");
                                }
                            }
                            libp2p::request_response::Event::<Box<[u8]>, ()>::OutboundFailure {
                                peer, error, ..
                            } => {
                                error!(peer = %peer, "Failed to send a message: {error}");
                            },
                            libp2p::request_response::Event::<Box<[u8]>, ()>::InboundFailure {peer, request_id, error} => {
                                warn!(peer = %peer, request_id = %request_id, "Failed to receive a message: {error}");
                            }
                            libp2p::request_response::Event::<Box<[u8]>, ()>::ResponseSent {..} => {
                                // trace!("Discarded messages not relevant for the protocol!");
                            },
                        }
                    }
                    SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::Acknowledgement(event)) => {
                        let _debug_span = tracing::span!(tracing::Level::DEBUG, "swarm protocol ACK", version = "0.1.0");
                        match event {
                            libp2p::request_response::Event::<Acknowledgement,()>::Message {
                                peer,
                                message
                            } => match message {
                                libp2p::request_response::Message::<Acknowledgement,()>::Request {
                                    request_id, request, channel
                                } => {
                                    trace!(peer = %peer, request_id = %request_id, "Received an acknowledgment");

                                    self.ack_received.unbounded_send((peer, request)).unwrap_or_else(|e| {
                                        error!(peer = %peer, request_id = %request_id, "Failed to process an acknowledgement: {e}");
                                    });

                                    if swarm.behaviour_mut().ack.send_response(channel, ()).is_err() {
                                        error!(peer = %peer, request_id = %request_id, "Failed to confirm receiving an acknowledgement, likely a timeout");
                                    };
                                },
                                libp2p::request_response::Message::<Acknowledgement,()>::Response {
                                    request_id, ..
                                } => {
                                    trace!(peer = %peer, request_id = %request_id, "Ack reception confirmed");
                                }
                            },
                            libp2p::request_response::Event::<Acknowledgement,()>::OutboundFailure {
                                peer, error, request_id
                            } => {
                                error!(peer = %peer, request_id = %request_id, "Failed to send an acknowledgement: {error}");
                            },
                            libp2p::request_response::Event::<Acknowledgement,()>::InboundFailure {peer, request_id, error} => {
                                warn!(peer = %peer, request_id = %request_id, "Failed to receive an acknowledgement: {error}");
                            }
                            libp2p::request_response::Event::<Acknowledgement,()>::ResponseSent {..} => {
                                // trace!("Discarded messages not relevant for the protocol!");
                            },
                        }
                    }
                    SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::TicketAggregation(event)) => {
                        let _debug_span = tracing::span!(tracing::Level::DEBUG, "swarm protocol TICKET_AGG", version = "0.1.0");
                        match event {
                            libp2p::request_response::Event::<Vec<legacy::AcknowledgedTicket>, std::result::Result<Ticket,String>>::Message {
                                peer,
                                message
                            } => {
                                match message {
                                    libp2p::request_response::Message::<Vec<legacy::AcknowledgedTicket>, std::result::Result<Ticket,String>>::Request {
                                        request_id, request, channel
                                    } => {
                                        trace!(peer = %peer, request_id = %request_id, "Received a ticket aggregation request");

                                        let request = request.into_iter().map(TransferableWinningTicket::from).collect::<Vec<_>>();
                                        if let Err(e) = aggregation_writer.receive_aggregation_request(peer, request, channel) {
                                            error!(peer = %peer, request_id = %request_id, "Failed to process a ticket aggregation request: {e}");
                                        }
                                    },
                                    libp2p::request_response::Message::<Vec<legacy::AcknowledgedTicket>, std::result::Result<Ticket,String>>::Response {
                                        request_id, response
                                    } => {
                                        if let Err(e) = aggregation_writer.receive_ticket(peer, response, request_id) {
                                            error!(peer = %peer, request_id = %request_id, "Failed to receive aggergated ticket: {e}");
                                        }
                                    }
                                }
                            },
                            libp2p::request_response::Event::<Vec<legacy::AcknowledgedTicket>, std::result::Result<Ticket,String>>::OutboundFailure {
                                peer, request_id, error,
                            } => {
                                error!(peer = %peer, request_id = %request_id, "Failed to send an aggregation request: {error}");
                            },
                            libp2p::request_response::Event::<Vec<legacy::AcknowledgedTicket>, std::result::Result<Ticket,String>>::InboundFailure {
                                peer, request_id, error
                            } => {
                                warn!(peer = %peer, request_id = %request_id, "Failed to receive an aggregated ticket: {error}");
                            },
                            libp2p::request_response::Event::<Vec<legacy::AcknowledgedTicket>, std::result::Result<Ticket,String>>::ResponseSent {..} => {
                                // trace!("Discarded messages not relevant for the protocol!");
                            },
                        }
                    }
                    SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::Heartbeat(event)) => {
                        let _debug_span = tracing::span!(tracing::Level::DEBUG, "swarm protocol HEARTBEAT", version = "0.1.0");
                        match event {
                            libp2p::request_response::Event::<Ping,Pong>::Message {
                                peer,
                                message
                            } => {
                                match message {
                                    libp2p::request_response::Message::<Ping,Pong>::Request {
                                        request_id, request, channel
                                    } => {
                                        trace!(peer = %peer, request_id = %request_id, "Received a heartbeat Ping");

                                        if let Ok(challenge_response) = ControlMessage::generate_pong_response(&request.0)
                                        {
                                            if swarm.behaviour_mut().heartbeat.send_response(channel, Pong(challenge_response, version.clone())).is_err() {
                                                error!(peer = %peer, request_id = %request_id, "Failed to reply to a Ping request");
                                            };
                                        }
                                    },
                                    libp2p::request_response::Message::<Ping,Pong>::Response {
                                        request_id, response
                                    } => {
                                        if let Some(replier) = active_pings.remove(&request_id) {
                                            trace!(peer = %peer, request_id = %request_id, "Processing manual ping response");
                                            replier.notify(response.0, response.1)
                                        } else {
                                            debug!(peer = %peer, request_id = %request_id, "Failed to find heartbeat replier");
                                        }
                                    }
                                }
                            },
                            libp2p::request_response::Event::<Ping,Pong>::OutboundFailure {
                                peer, request_id, error,
                            } => {
                                active_pings.remove(&request_id);
                                error!(peer = %peer, request_id = %request_id, "Failed to send a Pong reply: {error}");
                            },
                            libp2p::request_response::Event::<Ping,Pong>::InboundFailure {
                                peer, request_id, error
                            } => {
                                warn!(peer = %peer, request_id = %request_id, "Failed to receive a Pong request: {error}");
                            },
                            libp2p::request_response::Event::<Ping,Pong>::ResponseSent {..} => {
                                // trace!("Discarded messages not relevant for the protocol!");
                            },
                        }
                    }
                    SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::KeepAlive(_)) => {}
                    SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::Discovery(event)) => {
                        let _debug_span = tracing::span!(tracing::Level::DEBUG, "swarm behavior 'discovery'");

                        trace!(event = tracing::field::debug(&event), "Received a discovery event");
                        match event {
                            crate::behavior::discovery::Event::NewPeerMultiddress(peer, multiaddress) => {
                                info!(peer = %peer, multiaddress = %multiaddress, "New record");
                                swarm.behaviour_mut().heartbeat.add_address(&peer, multiaddress.clone());
                                swarm.behaviour_mut().msg.add_address(&peer, multiaddress.clone());
                                swarm.behaviour_mut().ack.add_address(&peer, multiaddress.clone());
                                swarm.behaviour_mut().ticket_aggregation.add_address(&peer, multiaddress.clone());

                                if let Err(e) = swarm.dial(multiaddress.clone()) {
                                    error!(peer = %peer, address = %multiaddress,  "Failed to dial the peer: {e}");
                                }
                            },
                        }
                    }
                    SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::TicketAggregationBehavior(event)) => {
                        let _debug_span = tracing::span!(tracing::Level::DEBUG, "swarm behavior 'ticket aggregation'");

                        trace!(event = tracing::field::debug(&event), "Received a discovery event");
                        match event {
                            TicketAggregationProcessed::Send(peer, acked_tickets, finalizer) => {
                                let ack_tkt_count = acked_tickets.len();
                                let request_id = swarm.behaviour_mut().ticket_aggregation.send_request(&peer, acked_tickets);
                                debug!(peer = %peer, request_id = %request_id, "Sending request to aggregate {ack_tkt_count} tickets");
                                active_aggregation_requests.insert(request_id, finalizer);
                            },
                            TicketAggregationProcessed::Reply(peer, ticket, response) => {
                                debug!(peer = %peer, "Enqueuing a response'");
                                if swarm.behaviour_mut().ticket_aggregation.send_response(response, ticket).is_err() {
                                    error!(peer = %peer, "Failed to enqueue response");
                                }
                            },
                            TicketAggregationProcessed::Receive(peer, acked_ticket, request) => {
                                on_ack_received.unbounded_send(acked_ticket).unwrap_or_else(|e| {
                                    error!(peer = %peer, request_id = %request, "Failed to process an aggregated acknowledgement: {e}");
                                });

                                match active_aggregation_requests.remove(&request) {
                                    Some(finalizer) => finalizer.finalize(),
                                    None => {
                                        warn!("transport input - ticket aggregation - response already handled")
                                    }
                                }
                            }
                        }
                    }
                    SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::HeartbeatGenerator(event)) => {
                        let _debug_span = tracing::span!(tracing::Level::DEBUG, "swarm behavior 'heartbeat generator'");

                        trace!(event = tracing::field::debug(&event), "Received a heartbeat event");
                        match event {
                            crate::behavior::heartbeat::Event::ToProbe((peer, replier)) => {
                                let req_id = swarm.behaviour_mut().heartbeat.send_request(&peer, Ping(replier.challenge()));
                                active_pings.insert(req_id, replier);
                            },
                        }
                    }
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
                    }
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
                    }
                    SwarmEvent::IncomingConnection {
                        connection_id,
                        local_addr,
                        send_back_addr,
                    } => {
                        trace!(local_addr = %local_addr, send_back_addr = %send_back_addr, connection_id = %connection_id, "transport - p2p - incoming connection");
                    }
                    SwarmEvent::IncomingConnectionError {
                        local_addr,
                        connection_id,
                        error,
                        send_back_addr,
                    } => {
                        error!(local_addr = %local_addr, send_back_addr = %send_back_addr, connection_id = %connection_id, "transport - p2p - incoming connection error: {error}")
                    }
                    SwarmEvent::OutgoingConnectionError {
                        connection_id,
                        error,
                        peer_id
                    } => {
                        error!(peer = tracing::field::debug(peer_id), connection_id = %connection_id, "transport - p2p - outgoing connection error: {error}")
                    }
                    SwarmEvent::NewListenAddr {
                        listener_id,
                        address,
                    } => {
                        debug!(listener_id = %listener_id, address = %address, "transport - p2p - new listen address")
                    }
                    SwarmEvent::ExpiredListenAddr {
                        listener_id,
                        address,
                    } => {
                        debug!(listener_id = %listener_id, address = %address, "transport - p2p - expired listen address")
                    }
                    SwarmEvent::ListenerClosed {
                        listener_id,
                        addresses,
                        reason,
                    } => {
                        debug!(listener_id = %listener_id, addresses = tracing::field::debug(addresses), "transport - p2p - listener closed: {reason:?}", )
                    }
                    SwarmEvent::ListenerError {
                        listener_id,
                        error,
                    } => {
                        debug!(listener_id = %listener_id, "transport - p2p - listener error: {error}")
                    }
                    SwarmEvent::Dialing {
                        peer_id,
                        connection_id,
                    } => {
                        debug!(peer = tracing::field::debug(peer_id), connection_id = %connection_id, "transport - p2p - dialing")
                    }
                    _ => error!("transport - p2p - unimplemented message type in p2p processing chain encountered")
                }
            }
        }
    }
}
