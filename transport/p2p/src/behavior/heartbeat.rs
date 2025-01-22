/// Behavior generating heartbeat requests for peers to be pinged.
///
/// This behavior is used with cooperation with the heartbeat request response protocol
/// and as such is used primarily as the source of peers to be pinged.
use std::{
    collections::VecDeque,
    task::{Context, Poll},
};

use futures::stream::{BoxStream, Stream, StreamExt};
use libp2p::{
    swarm::{dummy::ConnectionHandler, NetworkBehaviour, ToSwarm},
    PeerId,
};

use core_network::ping::PingQueryReplier;

#[derive(Debug)]
pub enum Event {
    ToProbe((PeerId, PingQueryReplier)),
}

pub struct Behaviour {
    events: BoxStream<'static, (PeerId, PingQueryReplier)>,
    pending_events: VecDeque<
        libp2p::swarm::ToSwarm<
            <Self as NetworkBehaviour>::ToSwarm,
            <<Self as NetworkBehaviour>::ConnectionHandler as libp2p::swarm::ConnectionHandler>::FromBehaviour,
        >,
    >,
}

impl Behaviour {
    pub fn new<T>(heartbeat_queue: T) -> Self
    where
        T: Stream<Item = (PeerId, PingQueryReplier)> + Send + 'static,
    {
        Self {
            events: Box::pin(heartbeat_queue),
            pending_events: VecDeque::new(),
        }
    }
}

impl NetworkBehaviour for Behaviour {
    type ConnectionHandler = ConnectionHandler;

    type ToSwarm = Event;

    fn handle_established_inbound_connection(
        &mut self,
        _connection_id: libp2p::swarm::ConnectionId,
        _peer: libp2p::PeerId,
        _local_addr: &libp2p::Multiaddr,
        _remote_addr: &libp2p::Multiaddr,
    ) -> Result<libp2p::swarm::THandler<Self>, libp2p::swarm::ConnectionDenied> {
        Ok(Self::ConnectionHandler {})
    }

    fn handle_established_outbound_connection(
        &mut self,
        _connection_id: libp2p::swarm::ConnectionId,
        _peer: libp2p::PeerId,
        _addr: &libp2p::Multiaddr,
        _role_override: libp2p::core::Endpoint,
        _port_use: libp2p::core::transport::PortUse,
    ) -> Result<libp2p::swarm::THandler<Self>, libp2p::swarm::ConnectionDenied> {
        Ok(Self::ConnectionHandler {})
    }

    fn on_swarm_event(&mut self, _event: libp2p::swarm::FromSwarm) {
        // No reaction to swarm event is necessary here, responses are handled by the protocol
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
        cx: &mut Context<'_>,
    ) -> Poll<libp2p::swarm::ToSwarm<Self::ToSwarm, libp2p::swarm::THandlerInEvent<Self>>> {
        if let Some(value) = self.pending_events.pop_front() {
            return Poll::Ready(value);
        };

        match self.events.poll_next_unpin(cx) {
            Poll::Ready(Some((peer_id, replier))) => {
                if let Some(value) = self.pending_events.pop_front() {
                    self.pending_events
                        .push_back(ToSwarm::GenerateEvent(Event::ToProbe((peer_id, replier))));
                    Poll::Ready(value)
                } else {
                    Poll::Ready(ToSwarm::GenerateEvent(Event::ToProbe((peer_id, replier))))
                }
            }
            Poll::Ready(None) | Poll::Pending => Poll::Pending,
        }
    }
}
