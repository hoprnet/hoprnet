//! The discovery mechanism uses an external stimulus to trigger the discovery
//! process on the libp2p side. It is responsible for processing the events
//! generated by other components and passing them to the libp2p swarm in
//! an appropriate format.
use std::collections::{HashMap, HashSet, VecDeque};

use futures::stream::{BoxStream, Stream, StreamExt};
use hopr_transport_protocol::PeerDiscovery;
use libp2p::{
    Multiaddr, PeerId,
    core::Endpoint,
    swarm::{
        CloseConnection, ConnectionDenied, ConnectionId, DialFailure, NetworkBehaviour, ToSwarm,
        dummy::ConnectionHandler,
    },
};

#[derive(Debug)]
pub enum DiscoveryInput {
    Indexer(PeerDiscovery),
}

#[derive(Debug)]
pub enum Event {
    IncomingConnection(PeerId, Multiaddr),
    FailedDial(PeerId),
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
    bootstrap_peers: HashMap<PeerId, Vec<Multiaddr>>,
    allowed_peers: HashSet<PeerId>,
    connected_peers: HashMap<PeerId, usize>,
}

impl Behaviour {
    pub fn new<T>(me: PeerId, onchain_events: T) -> Self
    where
        T: Stream<Item = PeerDiscovery> + Send + 'static,
    {
        Self {
            me,
            events: Box::pin(onchain_events.map(DiscoveryInput::Indexer)),
            bootstrap_peers: HashMap::new(),
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

    #[tracing::instrument(
        level = "debug",
        name = "Discovery::handle_established_inbound_connection",
        skip(self),
        fields(transport = "p2p discovery"),
        err(Display)
    )]
    fn handle_established_inbound_connection(
        &mut self,
        connection_id: libp2p::swarm::ConnectionId,
        peer: libp2p::PeerId,
        local_addr: &libp2p::Multiaddr,
        remote_addr: &libp2p::Multiaddr,
    ) -> Result<libp2p::swarm::THandler<Self>, libp2p::swarm::ConnectionDenied> {
        let is_allowed = self.allowed_peers.contains(&peer);
        tracing::trace!(%is_allowed, direction = "outbound", "Handling peer connection");

        if is_allowed {
            self.pending_events
                .push_back(ToSwarm::GenerateEvent(Event::IncomingConnection(
                    peer,
                    remote_addr.clone(),
                )));
        }

        is_allowed.then_some(Self::ConnectionHandler {}).ok_or_else(|| {
            libp2p::swarm::ConnectionDenied::new(crate::errors::P2PError::Logic(format!(
                "Connection from '{peer}' is not allowed"
            )))
        })
    }

    #[tracing::instrument(
        level = "debug",
        name = "Discovery::handle_pending_outbound_connection"
        skip(self),
        fields(transport = "p2p discovery"),
        ret(Debug),
        err(Display)
    )]
    fn handle_pending_outbound_connection(
        &mut self,
        connection_id: ConnectionId,
        maybe_peer: Option<PeerId>,
        addresses: &[Multiaddr],
        effective_role: Endpoint,
    ) -> Result<Vec<Multiaddr>, ConnectionDenied> {
        if let Some(peer) = maybe_peer {
            let is_allowed = self.allowed_peers.contains(&peer);
            tracing::trace!(%is_allowed, direction = "inbound", "Handling peer connection");

            if self.allowed_peers.contains(&peer) {
                // inject the multiaddress of the peer for possible dial usage by stream protocols
                return Ok(self
                    .bootstrap_peers
                    .get(&peer)
                    .map_or(vec![], |addresses| addresses.clone()));
            } else {
                return Err(libp2p::swarm::ConnectionDenied::new(crate::errors::P2PError::Logic(
                    format!("Connection to '{peer}' is not allowed"),
                )));
            }
        }

        Ok(vec![])
    }

    #[tracing::instrument(
        level = "trace",
        name = "Discovery::handle_established_outbound_connection",
        skip(self),
        fields(transport = "p2p discovery"),
        err(Display)
    )]
    fn handle_established_outbound_connection(
        &mut self,
        connection_id: libp2p::swarm::ConnectionId,
        peer: libp2p::PeerId,
        addr: &libp2p::Multiaddr,
        role_override: libp2p::core::Endpoint,
        port_use: libp2p::core::transport::PortUse,
    ) -> Result<libp2p::swarm::THandler<Self>, libp2p::swarm::ConnectionDenied> {
        // cannot connect without the handle_ending_outbound_connection being called first
        Ok(Self::ConnectionHandler {})
    }

    #[tracing::instrument(
        level = "debug",
        name = "Discovery::on_swarm_event"
        skip(self),
        fields(transport = "p2p discovery"),
    )]
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
            libp2p::swarm::FromSwarm::DialFailure(DialFailure { peer_id, error, .. }) => {
                tracing::debug!(?peer_id, %error, "Failed to dial peer");

                if let Some(peer) = peer_id {
                    self.pending_events
                        .push_back(ToSwarm::GenerateEvent(Event::FailedDial(peer)));
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

    #[tracing::instrument(
        level = "debug",
        name = "Discovery::poll"
        skip(self, cx),
        fields(transport = "p2p discovery")
    )]
    fn poll(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<libp2p::swarm::ToSwarm<Self::ToSwarm, libp2p::swarm::THandlerInEvent<Self>>> {
        if let Some(value) = self.pending_events.pop_front() {
            return std::task::Poll::Ready(value);
        };

        let poll_result = self.events.poll_next_unpin(cx).map(|e| match e {
            Some(DiscoveryInput::Indexer(event)) => match event {
                PeerDiscovery::Allow(peer) => {
                    let inserted_into_allow_list = self.allowed_peers.insert(peer);

                    let multiaddresses = self.bootstrap_peers.get(&peer);
                    if let Some(multiaddresses) = multiaddresses {
                        for address in multiaddresses {
                            self.pending_events.push_back(ToSwarm::NewExternalAddrOfPeer {
                                peer_id: peer,
                                address: address.clone(),
                            });
                        }
                    }

                    tracing::debug!(%peer, state = "allow", inserted_into_allow_list, emitted_libp2p_address_announce = multiaddresses.is_some_and(|v| !v.is_empty()), "Network registry");
                }
                PeerDiscovery::Ban(peer) => {
                    let was_allowed = self.allowed_peers.remove(&peer);
                    let is_connected = self.is_peer_connected(&peer);

                    if is_connected {
                        self.pending_events.push_back(ToSwarm::CloseConnection {
                            peer_id: peer,
                            connection: CloseConnection::default(),
                        });
                    }

                    tracing::debug!(%peer, state = "ban", was_allowed, will_close_active_connection = is_connected, "Network registry");
                }
                PeerDiscovery::Announce(peer, multiaddresses) => {
                    if peer != self.me {
                        tracing::debug!(%peer, addresses = ?&multiaddresses, "Announcement");

                        for multiaddress in &multiaddresses {
                            self.pending_events.push_back(ToSwarm::NewExternalAddrOfPeer {
                                peer_id: peer,
                                address: multiaddress.clone(),
                            });
                        }

                        self.bootstrap_peers.insert(peer, multiaddresses.clone());
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
