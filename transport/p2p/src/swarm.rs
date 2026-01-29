use std::num::NonZeroU8;
use std::sync::Arc;

use dashmap::DashSet;
use futures::{Stream, StreamExt};
use hopr_network_types::prelude::is_public_address;
use hopr_transport_identity::{
    Multiaddr,
    multiaddrs::{replace_transport_with_unspecified, resolve_dns_if_any},
};
use hopr_transport_protocol::PeerDiscovery;
use libp2p::{
    PeerId, autonat,
    identity::PublicKey,
    swarm::{NetworkInfo, SwarmEvent},
};
use tracing::{debug, error, info, trace, warn};

use crate::{HoprNetwork, HoprNetworkBehavior, HoprNetworkBehaviorEvent, constants, errors::Result};

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_TRANSPORT_P2P_OPEN_CONNECTION_COUNT:  hopr_metrics::SimpleGauge =  hopr_metrics::SimpleGauge::new(
        "hopr_transport_p2p_active_connection_count",
        "Number of currently active connections"
    ).unwrap();
    static ref METRIC_TRANSPORT_NAT_STATUS: hopr_metrics::SimpleGauge = hopr_metrics::SimpleGauge::new(
        "hopr_transport_p2p_nat_status",
        "Current NAT status as reported by libp2p autonat. 0=Unknown, 1=Public, 2=Private"
    ).unwrap();
    static ref METRIC_NETWORK_HEALTH: hopr_metrics::SimpleGauge =
         hopr_metrics::SimpleGauge::new("hopr_network_health", "Connectivity health indicator").unwrap();
}

pub struct InactiveNetwork {
    swarm: libp2p::Swarm<HoprNetworkBehavior>,
}

/// Build objects comprising an inactive p2p network.
///
/// Returns a built [libp2p::Swarm] object implementing the HoprNetworkBehavior functionality.
impl InactiveNetwork {
    #[cfg(feature = "runtime-tokio")]
    pub async fn build<T>(me: libp2p::identity::Keypair, external_discovery_events: T) -> Result<Self>
    where
        T: Stream<Item = PeerDiscovery> + Send + 'static,
    {
        let me_public: PublicKey = me.public();

        let swarm = libp2p::SwarmBuilder::with_existing_identity(me)
            .with_tokio()
            .with_tcp(
                libp2p::tcp::Config::default().nodelay(true),
                libp2p::noise::Config::new,
                // use default yamux configuration to enable auto-tuning
                // see https://github.com/libp2p/rust-libp2p/pull/4970
                libp2p::yamux::Config::default,
            )
            .map_err(|e| crate::errors::P2PError::Libp2p(e.to_string()))?;

        #[cfg(feature = "transport-quic")]
        let swarm = swarm.with_quic();

        let swarm = swarm.with_dns();

        Ok(Self {
            swarm: swarm
                .map_err(|e| crate::errors::P2PError::Libp2p(e.to_string()))?
                .with_behaviour(|_key| HoprNetworkBehavior::new(me_public, external_discovery_events))
                .map_err(|e| crate::errors::P2PError::Libp2p(e.to_string()))?
                .with_swarm_config(|cfg| {
                    cfg.with_dial_concurrency_factor(
                        NonZeroU8::new({
                            let v = std::env::var("HOPR_INTERNAL_LIBP2P_MAX_CONCURRENTLY_DIALED_PEER_COUNT")
                                .ok()
                                .and_then(|v| v.trim().parse::<u8>().ok())
                                .unwrap_or(constants::HOPR_SWARM_CONCURRENTLY_DIALED_PEER_COUNT);
                            v.max(1)
                        })
                        .expect("clamped to >= 1, will never fail"),
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
                .build(),
        })
    }

    pub fn with_listen_on(mut self, multiaddresses: Vec<Multiaddr>) -> Result<InactiveConfiguredNetwork> {
        for multiaddress in multiaddresses.iter() {
            match resolve_dns_if_any(multiaddress) {
                Ok(ma) => {
                    if let Err(e) = self.swarm.listen_on(ma.clone()) {
                        warn!(%multiaddress, listen_on=%ma, error = %e, "Failed to listen_on, will try to use an unspecified address");

                        match replace_transport_with_unspecified(&ma) {
                            Ok(ma) => {
                                if let Err(e) = self.swarm.listen_on(ma.clone()) {
                                    warn!(multiaddress = %ma, error = %e, "Failed to listen_on using the unspecified multiaddress",);
                                } else {
                                    info!(
                                        listen_on = ?ma,
                                        multiaddress = ?multiaddress,
                                        "Listening for p2p connections"
                                    );
                                    self.swarm.add_external_address(multiaddress.clone());
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
                        self.swarm.add_external_address(multiaddress.clone());
                    }
                }
                Err(error) => error!(%multiaddress, %error, "Failed to transform the multiaddress"),
            }
        }

        Ok(InactiveConfiguredNetwork { swarm: self.swarm })
    }
}

pub struct InactiveConfiguredNetwork {
    swarm: libp2p::Swarm<HoprNetworkBehavior>,
}

/// Builder of the network view and an actual background process running the libp2p core
/// event processing loop.
///
/// This object is primarily constructed to allow delayed starting of the background process,
/// as well as setup all the interconnections with the underlying network views to allow complex
/// functionality and signalling.
pub struct HoprLibp2pNetworkBuilder {
    pub(crate) swarm: libp2p::Swarm<HoprNetworkBehavior>,
    me: PeerId,
    my_addresses: Vec<Multiaddr>,
}

impl std::fmt::Debug for HoprLibp2pNetworkBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HoprSwarm").finish()
    }
}

impl From<HoprLibp2pNetworkBuilder> for libp2p::Swarm<HoprNetworkBehavior> {
    fn from(value: HoprLibp2pNetworkBuilder) -> Self {
        value.swarm
    }
}

impl HoprLibp2pNetworkBuilder {
    pub async fn new<T>(
        identity: libp2p::identity::Keypair,
        external_discovery_events: T,
        my_multiaddresses: Vec<Multiaddr>,
    ) -> Self
    where
        T: Stream<Item = PeerDiscovery> + Send + 'static,
    {
        #[cfg(all(feature = "prometheus", not(test)))]
        {
            METRIC_NETWORK_HEALTH.set(0.0);
        }

        let me = identity.public().to_peer_id();
        let swarm = InactiveNetwork::build(identity, external_discovery_events)
            .await
            .expect("swarm must be constructible");

        let swarm = swarm
            .with_listen_on(my_multiaddresses.clone())
            .expect("swarm must be configurable");

        Self {
            swarm: swarm.swarm,
            me,
            my_addresses: my_multiaddresses,
        }
    }

    pub fn into_network_with_stream_protocol_process(
        self,
        protocol: &'static str,
        allow_private_addresses: bool,
    ) -> (HoprNetwork, impl std::future::Future<Output = ()>) {
        let store =
            hopr_transport_network::store::NetworkPeerStore::new(self.me, self.my_addresses.into_iter().collect());

        let network = HoprNetwork {
            tracker: Arc::new(DashSet::new()),
            store: store.clone(),
            control: self.swarm.behaviour().streams.new_control(),
            protocol: libp2p::StreamProtocol::new(protocol),
        };

        let tracker = network.tracker.clone();
        #[cfg(all(feature = "prometheus", not(test)))]
        let network_inner = network.clone();
        let mut swarm = self.swarm;
        let process = async move {
            while let Some(event) = swarm.next().await {
                match event {
                    SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::Discovery(_)) => {}
                    SwarmEvent::Behaviour(
                        HoprNetworkBehaviorEvent::Autonat(event)
                    ) => {
                        match event {
                            autonat::Event::StatusChanged { old, new } => {
                                info!(?old, ?new, "AutoNAT status changed");
                                #[cfg(all(feature = "prometheus", not(test)))]
                                {
                                    let value = match new {
                                        autonat::NatStatus::Unknown => 0.0,
                                        autonat::NatStatus::Public(_) => 1.0,
                                        autonat::NatStatus::Private => 2.0,
                                    };
                                    METRIC_TRANSPORT_NAT_STATUS.set(value);
                                }
                            }
                            autonat::Event::InboundProbe { .. } => {}
                            autonat::Event::OutboundProbe { .. } => {}
                        }
                    }
                    SwarmEvent::Behaviour(HoprNetworkBehaviorEvent::Identify(_event)) => {}
                    SwarmEvent::ConnectionEstablished {
                        peer_id,
                        connection_id,
                        num_established,
                        established_in,
                        endpoint,
                        ..
                        // concurrent_dial_errors,
                    } => {
                        debug!(%peer_id, %connection_id, num_established, established_in_ms = established_in.as_millis(), transport="libp2p", "connection established");

                        if num_established == std::num::NonZero::<u32>::new(1).expect("must be a non-zero value") {
                            match endpoint {
                                libp2p::core::ConnectedPoint::Dialer { address, .. } => {
                                    if allow_private_addresses || is_public_address(&address) {
                                        if let Err(error) = store.add(peer_id, std::collections::HashSet::from([address])) {
                                            error!(peer = %peer_id, %error, direction = "outgoing", "failed to add connected peer to the peer store");
                                        }
                                    } else {
                                        debug!(transport="libp2p", peer = %peer_id, multiaddress = %address, "Private/local peer address encountered")
                                    }
                                    tracker.insert(peer_id);
                                },
                                libp2p::core::ConnectedPoint::Listener { send_back_addr, .. } => {
                                    if allow_private_addresses || is_public_address(&send_back_addr) {
                                        if let Err(error) = store.add(peer_id, std::collections::HashSet::from([send_back_addr])) {
                                            error!(peer = %peer_id, %error, direction = "incoming", "failed to add connected peer to the peer store");
                                        }
                                    } else {
                                        debug!(transport="libp2p", peer = %peer_id, multiaddress = %send_back_addr, "Private/local peer address ignored")
                                    }
                                    tracker.insert(peer_id);
                                }
                            }
                        } else {
                            trace!(transport="libp2p", peer = %peer_id, num_established, "Additional connection established")
                        }

                        print_network_info(swarm.network_info(), "connection established");

                        #[cfg(all(feature = "prometheus", not(test)))]
                        {
                            METRIC_NETWORK_HEALTH.set((hopr_api::network::NetworkView::health(&network_inner) as i32).into());
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

                        if num_established == 0 {
                            tracker.remove(&peer_id);
                        }

                        print_network_info(swarm.network_info(), "connection closed");

                        #[cfg(all(feature = "prometheus", not(test)))]
                        {
                            METRIC_NETWORK_HEALTH.set((hopr_api::network::NetworkView::health(&network_inner) as i32).into());
                            METRIC_TRANSPORT_P2P_OPEN_CONNECTION_COUNT.decrement(1.0);
                        }
                    }
                    SwarmEvent::IncomingConnection {
                        connection_id,
                        local_addr,
                        send_back_addr,
                    } => {
                        trace!(%local_addr, %send_back_addr, %connection_id, transport="libp2p", "incoming connection");
                    }
                    SwarmEvent::IncomingConnectionError {
                        local_addr,
                        connection_id,
                        error,
                        send_back_addr,
                        peer_id
                    } => {
                        debug!(?peer_id, %local_addr, %send_back_addr, %connection_id, transport="libp2p", %error, "incoming connection error");
                    }
                    SwarmEvent::OutgoingConnectionError {
                        connection_id,
                        error,
                        peer_id
                    } => {
                        debug!(peer = ?peer_id, %connection_id, transport="libp2p", %error, "outgoing connection error");

                        if let Some(peer_id) = peer_id
                            && !swarm.is_connected(&peer_id) {
                                if let Err(error) = store.remove(&peer_id) {
                                    error!(peer = %peer_id, %error, "failed to remove undialable peer from the peer store");
                                }
                                tracker.remove(&peer_id);
                            }

                        #[cfg(all(feature = "prometheus", not(test)))]
                        {
                            METRIC_NETWORK_HEALTH.set((hopr_api::network::NetworkView::health(&network_inner) as i32).into());
                        }
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
                    SwarmEvent::NewExternalAddrCandidate {address} => {
                        debug!(%address, "Detected new external address candidate")
                    }
                    SwarmEvent::ExternalAddrConfirmed { address } => {
                        info!(%address, "Detected external address")
                    }
                    SwarmEvent::ExternalAddrExpired {
                        ..  // address: Multiaddr
                    } => {}
                    SwarmEvent::NewExternalAddrOfPeer {
                        peer_id, address
                    } => {
                        // Only store public/routable addresses
                        if allow_private_addresses || is_public_address(&address) {
                            swarm.add_peer_address(peer_id, address.clone());
                            trace!(transport="libp2p", peer = %peer_id, multiaddress = %address, "Public peer address stored in swarm")
                        } else {
                            trace!(transport="libp2p", peer = %peer_id, multiaddress = %address, "Private/local peer address ignored")
                        }
                    },
                    _ => trace!(transport="libp2p", "Unsupported enum option detected")
                }
            }
        };

        (network, process)
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
