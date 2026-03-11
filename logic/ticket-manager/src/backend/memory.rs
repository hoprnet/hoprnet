use hopr_api::{
    chain::{ChannelId, RedeemableTicket},
    db::ChannelTicketStatistics,
};

use crate::{
    OutgoingIndexStore, TicketManagerError, TicketStatsStore,
    traits::{TicketQueue, TicketQueueStore},
};

/// Simple non-persistent ticket queue store backed by a `dashmap::DashMap` and [`MemoryTicketQueue`].
///
/// Useful for non-persistent and testing scenarios.
#[derive(Clone, Debug, Default)]
pub struct MemoryStore(dashmap::DashMap<ChannelId, MemoryTicketQueue>);

impl TicketQueueStore for MemoryStore {
    type Queue = MemoryTicketQueue;

    fn open_or_create(&self, channel_id: &ChannelId) -> Result<Self::Queue, <Self::Queue as TicketQueue>::Error> {
        Ok(self
            .0
            .entry(*channel_id)
            .or_insert_with(MemoryTicketQueue::default)
            .clone())
    }

    fn iter_channels(&self) -> impl Iterator<Item = ChannelId> {
        self.0.iter().map(|e| e.key().clone())
    }
}

impl TicketStatsStore for MemoryStore {
    type Error = std::convert::Infallible;

    fn load_stats(&self) -> Result<ChannelTicketStatistics, TicketManagerError> {
        todo!()
    }

    fn save_stats(&self, stats: ChannelTicketStatistics) -> Result<(), TicketManagerError> {
        todo!()
    }
}

impl OutgoingIndexStore for MemoryStore {
    type Error = std::convert::Infallible;

    fn load_outgoing_index(&self, channel_id: &ChannelId) -> Result<u64, TicketManagerError> {
        todo!()
    }

    fn save_outgoing_index(&self, channel_id: &ChannelId, index: u64) -> Result<(), TicketManagerError> {
        todo!()
    }
}

/// Simple in-memory ticket queue implementation using a binary heap.
///
/// This is suitable for testing where ticket persistence is not required.
#[derive(Clone, Debug, Default)]
pub struct MemoryTicketQueue(std::collections::BinaryHeap<RedeemableTicket>);

impl TicketQueue for MemoryTicketQueue {
    type Error = std::convert::Infallible;

    fn len(&self) -> usize {
        self.0.len()
    }

    fn push(&mut self, ticket: RedeemableTicket) -> Result<(), Self::Error> {
        Ok(self.0.push(ticket))
    }

    fn pop(&mut self) -> Result<Option<RedeemableTicket>, Self::Error> {
        Ok(self.0.pop())
    }

    fn peek(&self) -> Result<Option<RedeemableTicket>, Self::Error> {
        Ok(self.0.peek().cloned())
    }

    fn iter_unordered(&self) -> impl Iterator<Item = Result<RedeemableTicket, Self::Error>> {
        self.0.iter().cloned().map(Ok)
    }
}
