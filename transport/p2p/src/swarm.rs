use futures::{select, Stream, StreamExt};
use hopr_transport_network::messaging::ControlMessage;
use libp2p::autonat;
use libp2p::swarm::SwarmEvent;
use libp2p::swarm::{dial_opts::DialOpts, NetworkInfo};
use libp2p::{multiaddr::Protocol, request_response::OutboundRequestId, request_response::ResponseChannel};
use std::net::Ipv4Addr;
use std::num::NonZeroU8;
use tracing::{debug, error, info, trace, warn};

use hopr_internal_types::prelude::*;
use hopr_transport_identity::{
    multiaddrs::{replace_transport_with_unspecified, resolve_dns_if_any},
    Multiaddr, PeerId,
};
use hopr_transport_network::{network::NetworkTriggeredEvent, ping::PingQueryReplier};
use hopr_transport_protocol::{config::ProtocolConfig, PeerDiscovery};

use crate::{constants, errors::Result, HoprNetworkBehavior};
use crate::{HoprNetworkBehaviorEvent, Ping, Pong};

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
async fn build_p2p_network<T>(
    me: libp2p::identity::Keypair,
    network_update_input: futures::channel::mpsc::Receiver<NetworkTriggeredEvent>,
    indexer_update_input: T,
    heartbeat_requests: futures::channel::mpsc::UnboundedReceiver<(PeerId, PingQueryReplier)>,
    protocol_cfg: ProtocolConfig,
) -> Result<libp2p::Swarm<HoprNetworkBehavior>>
where
    T: Stream<Item = PeerDiscovery> + Send + 'static,
{
    let me_peerid: PeerId = me.public().into();

    // Both features could be enabled during testing, therefore we only use tokio when its
    // exclusively enabled.
    #[cfg(feature = "runtime-tokio")]
    let swarm = libp2p::SwarmBuilder::with_existing_identity(me)
        .with_tokio()
        .with_tcp(
            libp2p::tcp::Config::default().nodelay(true),
            libp2p::noise::Config::new,
            // use default yamux configuration to enable auto-tuning
            // see https://github.com/libp2p/rust-libp2p/pull/4970
            libp2p::yamux::Config::default,
        )
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
                protocol_cfg.heartbeat.timeout,
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
                    .and_then(|v| v.parse::<usize>().map_err(|_e| std::env::VarError::NotPresent))
                    .unwrap_or(constants::HOPR_SWARM_CONCURRENTLY_NEGOTIATING_INBOUND_PEER_COUNT),
            )
            .with_idle_connection_timeout(
                std::env::var("HOPR_INTERNAL_LIBP2P_SWARM_IDLE_TIMEOUT")
                    .and_then(|v| v.parse::<u64>().map_err(|_e| std::env::VarError::NotPresent))
                    .map(std::time::Duration::from_secs)
                    .unwrap_or(constants::HOPR_SWARM_IDLE_CONNECTION_TIMEOUT),
            )
        })
        .build())
}

pub struct HoprSwarm {
    pub(crate) swarm: libp2p::Swarm<HoprNetworkBehavior>,
}

impl std::fmt::Debug for HoprSwarm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HoprSwarm").finish()
    }
}

impl From<HoprSwarm> for libp2p::Swarm<HoprNetworkBehavior> {
    fn from(value: HoprSwarm) -> Self {
        value.swarm
    }
}

impl HoprSwarm {
    pub async fn new<T>(
        identity: libp2p::identity::Keypair,
        network_update_input: futures::channel::mpsc::Receiver<NetworkTriggeredEvent>,
        indexer_update_input: T,
        heartbeat_requests: futures::channel::mpsc::UnboundedReceiver<(PeerId, PingQueryReplier)>,
        my_multiaddresses: Vec<Multiaddr>,
        protocol_cfg: ProtocolConfig,
    ) -> Self
    where
        T: Stream<Item = PeerDiscovery> + Send + 'static,
    {
        let mut swarm = build_p2p_network(
            identity,
            network_update_input,
            indexer_update_input,
            heartbeat_requests,
            protocol_cfg,
        )
        .await
        .expect("swarm must be constructible");

        for multiaddress in my_multiaddresses.iter() {
            match resolve_dns_if_any(multiaddress) {
                Ok(ma) => {
                    if let Err(e) = swarm.listen_on(ma.clone()) {
                        warn!(%multiaddress, listen_on=%ma, error = %e, "Failed to listen_on, will try to use an unspecified address");

                        match replace_transport_with_unspecified(&ma) {
                            Ok(ma) => {
                                if let Err(e) = swarm.listen_on(ma.clone()) {
                                    warn!(multiaddress = %ma, error = %e, "Failed to listen_on using the unspecified multiaddress",);
                                } else {
                                    info!(
                                        listen_on = ?ma,
                                        multiaddress = ?multiaddress,
                                        "Listening for p2p connections"
                                    );
                                    swarm.add_external_address(multiaddress.clone());
                                }
                            }
                            Err(e) => {
                                error!(multiaddress = %ma, error = %e, "Failed to transform the multiaddress")
                            }
                        }
                    } else {
                        info!(
                            listen_on = ?ma,
                            multiaddress = ?multiaddress,
                            "Listening for p2p connections"
                        );
                        swarm.add_external_address(multiaddress.clone());
                    }
                }
                Err(e) => error!(%multiaddress, error = %e, "Failed to transform the multiaddress"),
            }
        }

        // TODO: perform this check
        // NOTE: This would be a valid check but is not immediate
        // assert!(
        //     swarm.listeners().count() > 0,
        //     "The node failed to listen on at least one of the specified interfaces"
        // );

        Self { swarm }
    }

    pub fn build_protocol_control(&self, protocol: &'static str) -> crate::HoprStreamProtocolControl {
        crate::HoprStreamProtocolControl::new(self.swarm.behaviour().streams.new_control(), protocol)
    }

    /// Main p2p loop that instantiates a new libp2p::Swarm instance and sets up listening and reacting pipelines
    /// running in a neverending loop future.
    ///
    /// The function represents the entirety of the business logic of the hopr daemon related to core operations.
    ///
    /// This future can only be resolved by an unrecoverable error or a panic.
    pub async fn run(self, version: String) {
        let mut swarm: libp2p::Swarm<HoprNetworkBehavior> = self.into();

        // NOTE: an improvement would be a forgetting cache for the active requests
        let active_pings: moka::future::Cache<libp2p::request_response::OutboundRequestId, PingQueryReplier> =
            moka::future::CacheBuilder::new(1000)
                .time_to_live(std::time::Duration::from_secs(40))
                .build();

        loop {
            select! {
                event = swarm.select_next_some() => match event {
                    SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::Heartbeat(event)) => {
                        let _span = tracing::span!(tracing::Level::DEBUG, "swarm protocol", protocol = "/hopr/heartbeat/0.1.0");
                        match event {
                            libp2p::request_response::Event::<Ping,Pong>::Message {
                                peer,
                                message,
                                connection_id
                            } => {
                                match message {
                                    libp2p::request_response::Message::<Ping,Pong>::Request {
                                        request_id, request, channel
                                    } => {
                                        trace!(%peer, %request_id, %connection_id, "Received a heartbeat Ping");

                                        if let Ok(challenge_response) = ControlMessage::generate_pong_response(&request.0)
                                        {
                                            if swarm.behaviour_mut().heartbeat.send_response(channel, Pong(challenge_response, version.clone())).is_err() {
                                                error!(%peer, %request_id, %connection_id, "Failed to reply to a Ping request");
                                            };
                                        }
                                    },
                                    libp2p::request_response::Message::<Ping,Pong>::Response {
                                        request_id, response
                                    } => {
                                        if let Some(replier) = active_pings.remove(&request_id).await {
                                            active_pings.run_pending_tasks().await;     // needed to remove the invalidated, but still present instance of Arc inside
                                            trace!(%peer, %request_id, "Processing manual ping response");
                                            replier.notify(response.0, response.1)
                                        } else {
                                            debug!(%peer, %request_id, "Failed to find heartbeat replier");
                                        }
                                    }
                                }
                            },
                            libp2p::request_response::Event::<Ping,Pong>::OutboundFailure {
                                peer, request_id, error, connection_id
                            } => {
                                active_pings.invalidate(&request_id).await;
                                if matches!(error, libp2p::request_response::OutboundFailure::DialFailure) {
                                    trace!(%peer, %request_id, %connection_id, %error, "Peer is offline");
                                } else {
                                    error!(%peer, %request_id, %connection_id, %error, "Failed heartbeat protocol on outbound");
                                }
                            },
                            libp2p::request_response::Event::<Ping,Pong>::InboundFailure {
                                peer, request_id, error, connection_id
                            } => {
                                warn!(%peer, %request_id, %connection_id, "Failed to receive a Pong request: {error}");
                            },
                            libp2p::request_response::Event::<Ping,Pong>::ResponseSent {..} => {
                                // trace!("Discarded messages not relevant for the protocol!");
                            },
                        }
                    }
                    SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::KeepAlive(_)) => {}
                    SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::Discovery(_)) => {}
                    SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::AutonatClient(autonat::v2::client::Event {
                        server,
                        tested_addr,
                        bytes_sent,
                        result,
                    })) => {
                        match result {
                            Ok(_) => {
                                debug!(%server, %tested_addr, %bytes_sent, "Autonat server successfully tested");
                            }
                            Err(e) => {
                                warn!(%server, %tested_addr, %bytes_sent, %e, "Autonat server test failed");
                            }
                        }
                    }
                    SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::AutonatServer(event)) => {
                        warn!(?event, "Autonat server event");
                    }

                    SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::HeartbeatGenerator(event)) => {
                        let _span = tracing::span!(tracing::Level::DEBUG, "swarm behavior", behavior="heartbeat generator");

                        trace!(event = tracing::field::debug(&event), "Received a heartbeat event");
                        match event {
                            crate::behavior::heartbeat::Event::ToProbe((peer, replier)) => {
                                let req_id = swarm.behaviour_mut().heartbeat.send_request(&peer, Ping(replier.challenge()));
                                active_pings.insert(req_id, replier).await;
                            },
                        }
                    }
                    SwarmEvent::ConnectionEstablished {
                        peer_id,
                        connection_id,
                        num_established,
                        established_in,
                        ..
                        // concurrent_dial_errors,
                        // endpoint,
                    } => {
                        debug!(%peer_id, %connection_id, num_established, established_in_ms = established_in.as_millis(), transport="libp2p", "connection established");

                        print_network_info(swarm.network_info(), "connection established");

                        #[cfg(all(feature = "prometheus", not(test)))]
                        {
                            METRIC_TRANSPORT_P2P_OPEN_CONNECTION_COUNT.increment(1.0);
                        }
                    }
                    SwarmEvent::ConnectionClosed {
                        peer_id,
                        connection_id,
                        cause,
                        num_established,
                        ..
                        // endpoint,
                    } => {
                        debug!(%peer_id, %connection_id, num_established, transport="libp2p", "connection closed: {cause:?}");

                        print_network_info(swarm.network_info(), "connection closed");

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
                        trace!(%local_addr, %send_back_addr, %connection_id, transport="libp2p",  "incoming connection");
                    }
                    SwarmEvent::IncomingConnectionError {
                        local_addr,
                        connection_id,
                        error,
                        send_back_addr,
                    } => {
                        error!(%local_addr, %send_back_addr, %connection_id, transport="libp2p", %error, "incoming connection error");

                        print_network_info(swarm.network_info(), "incoming connection error");
                    }
                    SwarmEvent::OutgoingConnectionError {
                        connection_id,
                        error,
                        peer_id
                    } => {
                        error!(peer = ?peer_id, %connection_id, transport="libp2p", %error, "outgoing connection error");

                        print_network_info(swarm.network_info(), "outgoing connection error");
                    }
                    SwarmEvent::NewListenAddr {
                        listener_id,
                        address,
                    } => {
                        debug!(%listener_id, %address, transport="libp2p", "new listen address")
                    }
                    SwarmEvent::ExpiredListenAddr {
                        listener_id,
                        address,
                    } => {
                        debug!(%listener_id, %address, transport="libp2p", "expired listen address")
                    }
                    SwarmEvent::ListenerClosed {
                        listener_id,
                        addresses,
                        reason,
                    } => {
                        debug!(%listener_id, ?addresses, ?reason, transport="libp2p", "listener closed", )
                    }
                    SwarmEvent::ListenerError {
                        listener_id,
                        error,
                    } => {
                        debug!(%listener_id, transport="libp2p", %error, "listener error")
                    }
                    SwarmEvent::Dialing {
                        peer_id,
                        connection_id,
                    } => {
                        debug!(peer = ?peer_id, %connection_id, transport="libp2p", "dialing")
                    }
                    SwarmEvent::NewExternalAddrCandidate {
                        ..  // address: Multiaddr
                    } => {}
                    SwarmEvent::ExternalAddrConfirmed { address } => {
                        info!(%address, "Detected external address")
                    }
                    SwarmEvent::ExternalAddrExpired {
                        ..  // address: Multiaddr
                    } => {}
                    SwarmEvent::NewExternalAddrOfPeer {
                        peer_id, address
                    } => {
                        trace!(transport="libp2p", peer = %peer_id, multiaddress = %address, "New peer stored in swarm")
                    },
                    _ => trace!(transport="libp2p", "Unsupported enum option detected")
                }
            }
        }
    }

    pub fn run_nat_server(&mut self, port: u16) {
        info!(listen_on = port, "Starting NAT server");

        match self.swarm.listen_on(
            Multiaddr::empty()
                .with(Protocol::Ip4(Ipv4Addr::UNSPECIFIED))
                .with(Protocol::Tcp(port)),
        ) {
            Ok(_) => {
                info!("NAT server started");
            }
            Err(e) => {
                warn!(error = %e, "Failed to listen on NAT server");
            }
        }
    }
    pub fn dial_nat_server(&mut self, addresses: Vec<Multiaddr>) {
        // let dial_opts = DialOpts::peer_id(PeerId::random())
        //     .addresses(addresses)
        //     .extend_addresses_through_behaviour()
        //     .build();
        info!(
            num_addresses = addresses.len(),
            "Dialing NAT servers with multiple candidate addresses"
        );

        for addr in addresses {
            let dial_opts = DialOpts::unknown_peer_id().address(addr.clone()).build();
            if let Err(e) = self.swarm.dial(dial_opts) {
                warn!(%addr, %e, "Failed to dial NAT server address");
            } else {
                info!(%addr, "Dialed NAT server address");
                break;
            }
        }
    }
}

fn print_network_info(network_info: NetworkInfo, event: &str) {
    let num_peers = network_info.num_peers();
    let connection_counters = network_info.connection_counters();
    let num_incoming = connection_counters.num_established_incoming();
    let num_outgoing = connection_counters.num_established_outgoing();
    info!(
        num_peers,
        num_incoming, num_outgoing, "swarm network status after {event}"
    );
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

pub type TicketAggregationRequestType = OutboundRequestId;
pub type TicketAggregationResponseType = ResponseChannel<std::result::Result<Ticket, String>>;
