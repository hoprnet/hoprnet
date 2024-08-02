/// TODO: Add discovery documentation here
use std::collections::{HashMap, HashSet, VecDeque};

use futures::stream::{BoxStream, Stream, StreamExt};
use futures_concurrency::stream::Merge;
use libp2p::{
    swarm::{dummy::ConnectionHandler, CloseConnection, NetworkBehaviour, ToSwarm},
    Multiaddr, PeerId,
};

use core_network::network::NetworkTriggeredEvent;
use tracing::debug;

use crate::PeerDiscovery;

#[derive(Debug)]
pub enum DiscoveryInput {
    NetworkUpdate(NetworkTriggeredEvent),
    Indexer(PeerDiscovery),
}

#[derive(Debug)]
pub enum Event {
    NewPeerMultiddress(PeerId, Multiaddr),
}

pub struct Behaviour {
    me: PeerId,
    events: BoxStream<'static, DiscoveryInput>,
    pending_events: VecDeque<
        libp2p::swarm::ToSwarm<
            <Self as NetworkBehaviour>::ToSwarm,
            <<Self as NetworkBehaviour>::ConnectionHandler as libp2p::swarm::ConnectionHandler>::FromBehaviour,
        >,
    >,
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
            pending_events: VecDeque::new(),
            allowed_peers: HashSet::new(),
            connected_peers: HashMap::new(),
        }
    }

    fn is_peer_connected(&self, peer: &PeerId) -> bool {
        if let Some(connection_count) = self.connected_peers.get(peer) {
            if *connection_count > 0 {
                return true;
            }
        }

        false
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
                *self.connected_peers.entry(data.peer_id).or_insert(0) -= 1
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
                        for multiaddress in multiaddresses.into_iter() {
                            self.pending_events
                                .push_back(ToSwarm::GenerateEvent(Event::NewPeerMultiddress(peer, multiaddress)));
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
