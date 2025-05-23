/// TODO: Add discovery documentation here
use std::collections::{HashMap, HashSet, VecDeque};

use futures::stream::{BoxStream, Stream, StreamExt};
use futures_concurrency::stream::Merge;
use hopr_transport_network::network::NetworkTriggeredEvent;
use hopr_transport_protocol::PeerDiscovery;
use libp2p::{
    Multiaddr, PeerId,
    swarm::{CloseConnection, NetworkBehaviour, ToSwarm, dial_opts::DialOpts, dummy::ConnectionHandler},
};
use tracing::debug;

#[derive(Debug)]
pub enum DiscoveryInput {
    NetworkUpdate(NetworkTriggeredEvent),
    Indexer(PeerDiscovery),
}

#[derive(Debug)]
pub enum Event {}

pub struct Behaviour {
    me: PeerId,
    events: BoxStream<'static, DiscoveryInput>,
    pending_events: VecDeque<
        libp2p::swarm::ToSwarm<
            <Self as NetworkBehaviour>::ToSwarm,
            <<Self as NetworkBehaviour>::ConnectionHandler as libp2p::swarm::ConnectionHandler>::FromBehaviour,
        >,
    >,
    all_peers: HashMap<PeerId, Multiaddr>,
    allowed_peers: HashSet<PeerId>,
    connected_peers: HashMap<PeerId, usize>,
}

impl Behaviour {
    pub fn new<T, U>(me: PeerId, network_events: T, onchain_events: U) -> Self
    where
        T: Stream<Item = NetworkTriggeredEvent> + Send + 'static,
        U: Stream<Item = PeerDiscovery> + Send + 'static,
    {
        Self {
            me,
            events: Box::pin(
                (
                    network_events.map(DiscoveryInput::NetworkUpdate),
                    onchain_events.map(DiscoveryInput::Indexer),
                )
                    .merge()
                    .fuse(),
            ),
            all_peers: HashMap::new(),
            pending_events: VecDeque::new(),
            allowed_peers: HashSet::new(),
            connected_peers: HashMap::new(),
        }
    }

    fn is_peer_connected(&self, peer: &PeerId) -> bool {
        self.connected_peers.get(peer).map(|v| *v > 0).unwrap_or(false)
    }
}

impl NetworkBehaviour for Behaviour {
    type ConnectionHandler = ConnectionHandler;
    type ToSwarm = Event;

    fn handle_established_inbound_connection(
        &mut self,
        _connection_id: libp2p::swarm::ConnectionId,
        peer: libp2p::PeerId,
        _local_addr: &libp2p::Multiaddr,
        _remote_addr: &libp2p::Multiaddr,
    ) -> Result<libp2p::swarm::THandler<Self>, libp2p::swarm::ConnectionDenied> {
        if self.allowed_peers.contains(&peer) {
            Ok(Self::ConnectionHandler {})
        } else {
            Err(libp2p::swarm::ConnectionDenied::new(crate::errors::P2PError::Logic(
                format!("Connection from '{peer}' is not allowed"),
            )))
        }
    }

    fn handle_established_outbound_connection(
        &mut self,
        _connection_id: libp2p::swarm::ConnectionId,
        peer: libp2p::PeerId,
        _addr: &libp2p::Multiaddr,
        _role_override: libp2p::core::Endpoint,
        _port_use: libp2p::core::transport::PortUse,
    ) -> Result<libp2p::swarm::THandler<Self>, libp2p::swarm::ConnectionDenied> {
        if self.allowed_peers.contains(&peer) {
            Ok(Self::ConnectionHandler {})
        } else {
            Err(libp2p::swarm::ConnectionDenied::new(crate::errors::P2PError::Logic(
                format!("Connection to '{peer}' is not allowed"),
            )))
        }
    }

    fn on_swarm_event(&mut self, event: libp2p::swarm::FromSwarm) {
        match event {
            libp2p::swarm::FromSwarm::ConnectionEstablished(data) => {
                *self.connected_peers.entry(data.peer_id).or_insert(0) += 1
            }
            libp2p::swarm::FromSwarm::ConnectionClosed(data) => {
                let v = self.connected_peers.entry(data.peer_id).or_insert(0);
                if *v > 0 {
                    *v -= 1;
                };
            }
            libp2p::swarm::FromSwarm::DialFailure(failure) => {
                // NOTE: libp2p swarm in the current version removes the (PeerId, Multiaddr) from the cache on a dial
                // failure, therefore it needs to be readded back to the swarm on every dial failure,
                // for now we want to mirror the entire announcement back to the swarm
                if let Some(peer_id) = failure.peer_id {
                    if let Some(multiaddress) = self.all_peers.get(&peer_id) {
                        self.pending_events.push_back(ToSwarm::NewExternalAddrOfPeer {
                            peer_id,
                            address: multiaddress.clone(),
                        });
                    }
                }
            }
            _ => {}
        }
    }

    fn on_connection_handler_event(
        &mut self,
        _peer_id: libp2p::PeerId,
        _connection_id: libp2p::swarm::ConnectionId,
        _event: libp2p::swarm::THandlerOutEvent<Self>,
    ) {
        // Nothing is necessary here, because no ConnectionHandler events should be generated
    }

    fn poll(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<libp2p::swarm::ToSwarm<Self::ToSwarm, libp2p::swarm::THandlerInEvent<Self>>> {
        if let Some(value) = self.pending_events.pop_front() {
            return std::task::Poll::Ready(value);
        };

        let poll_result = self.events.poll_next_unpin(cx).map(|e| match e {
            Some(DiscoveryInput::NetworkUpdate(event)) => match event {
                NetworkTriggeredEvent::CloseConnection(peer) => {
                    debug!(peer = %peer, "p2p - discovery - Closing connection (reason: low ping connection quality");
                    if self.is_peer_connected(&peer) {
                        self.pending_events.push_back(ToSwarm::CloseConnection {
                            peer_id: peer,
                            connection: CloseConnection::default(),
                        });
                    }
                }
                NetworkTriggeredEvent::UpdateQuality(_, _) => {}
            },
            Some(DiscoveryInput::Indexer(event)) => match event {
                PeerDiscovery::Allow(peer) => {
                    debug!(peer = %peer, "p2p - discovery - Network registry allow");
                    let _ = self.allowed_peers.insert(peer);

                    if let Some(multiaddress) = self.all_peers.get(&peer) {
                        self.pending_events.push_back(ToSwarm::NewExternalAddrOfPeer {
                            peer_id: peer,
                            address: multiaddress.clone(),
                        });
                    }
                }
                PeerDiscovery::Ban(peer) => {
                    debug!(peer = %peer, "p2p - discovery - Network registry ban");
                    self.allowed_peers.remove(&peer);

                    if self.is_peer_connected(&peer) {
                        debug!(peer = %peer, "p2p - discovery - Requesting disconnect due to ban");
                        self.pending_events.push_back(ToSwarm::CloseConnection {
                            peer_id: peer,
                            connection: CloseConnection::default(),
                        });
                    }
                }
                PeerDiscovery::Announce(peer, multiaddresses) => {
                    if peer != self.me {
                        debug!(peer = %peer, addresses = tracing::field::debug(&multiaddresses), "p2p - discovery - Announcement");
                        if let Some(multiaddress) = multiaddresses.last() {
                            self.all_peers.insert(peer, multiaddress.clone());

                            self.pending_events.push_back(ToSwarm::NewExternalAddrOfPeer {
                                peer_id: peer,
                                address: multiaddress.clone(),
                            });

                            // the dial is important to create a first connection some time before the heartbeat mechanism
                            // kicks in, otherwise the heartbeat is likely to fail on the first try due to dial and protocol
                            // negotiation taking longer than the request response timeout
                            self.pending_events.push_back(ToSwarm::Dial { opts: DialOpts::peer_id(peer).addresses(multiaddresses).build()});
                        }
                    }
                }
            },
            None => {}
        });

        if matches!(poll_result, std::task::Poll::Pending) {
            std::task::Poll::Pending
        } else if let Some(value) = self.pending_events.pop_front() {
            std::task::Poll::Ready(value)
        } else {
            std::task::Poll::Pending
        }
    }
}
