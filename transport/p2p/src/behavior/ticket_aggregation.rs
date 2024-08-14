/// Behavior emitting the ticket aggregation processor events used to trigger the ticket
/// aggregation request response protocol.
///
/// This behavior is used with cooperation with the ticket aggregation request response protocol
/// and as such is used primarily as the source of events to be performed with ticket aggregation
/// on the wire level.
use std::{
    collections::VecDeque,
    task::{Context, Poll},
};

use futures::stream::{BoxStream, Stream, StreamExt};
use libp2p::swarm::{dummy::ConnectionHandler, NetworkBehaviour, ToSwarm};

use hopr_transport_protocol::ticket_aggregation::processor::TicketAggregationProcessed;

use crate::swarm::{TicketAggregationRequestType, TicketAggregationResponseType};

pub type Event = TicketAggregationProcessed<TicketAggregationResponseType, TicketAggregationRequestType>;

pub struct Behaviour {
    events: BoxStream<'static, Event>,
    pending_events: VecDeque<
        libp2p::swarm::ToSwarm<
            <Self as NetworkBehaviour>::ToSwarm,
            <<Self as NetworkBehaviour>::ConnectionHandler as libp2p::swarm::ConnectionHandler>::FromBehaviour,
        >,
    >,
}

impl Behaviour {
    pub fn new<T>(ticket_aggregation_events: T) -> Self
    where
        T: Stream<Item = Event> + Send + 'static,
    {
        Self {
            events: Box::pin(ticket_aggregation_events),
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
            std::task::Poll::Ready(Some(ticket_agg_event)) => {
                self.pending_events.push_back(ToSwarm::GenerateEvent(ticket_agg_event));
                std::task::Poll::Ready(self.pending_events.pop_front().unwrap())
            }
            std::task::Poll::Ready(None) | std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}
