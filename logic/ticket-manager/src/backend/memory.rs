use hopr_api::{
    chain::{ChannelId, RedeemableTicket},
    db::ChannelTicketStatistics,
};

use crate::{
    OutgoingIndexStore, TicketManagerError,
    traits::{TicketQueue, TicketQueueStore},
};

/// Simple non-persistent ticket queue store backed by a `HashMap` and [`MemoryTicketQueue`].
///
/// Useful for non-persistent and testing scenarios.
#[derive(Clone, Debug, Default)]
pub struct MemoryStore {
    tickets: std::collections::HashMap<ChannelId, MemoryTicketQueue>,
    out_indices: std::collections::HashMap<(ChannelId, u32), u64>,
}

impl TicketQueueStore for MemoryStore {
    type Queue = MemoryTicketQueue;

    fn open_or_create_queue(
        &mut self,
        channel_id: &ChannelId,
    ) -> Result<Self::Queue, <Self::Queue as TicketQueue>::Error> {
        Ok(self
            .tickets
            .entry(*channel_id)
            .or_insert_with(MemoryTicketQueue::default)
            .clone())
    }

    fn delete_queue(&mut self, channel_id: &ChannelId) -> Result<(), <Self::Queue as TicketQueue>::Error> {
        self.tickets.remove(channel_id);
        Ok(())
    }

    fn iter_queues(&self) -> impl Iterator<Item = ChannelId> {
        self.tickets.iter().map(|(k, _)| *k)
    }
}

impl OutgoingIndexStore for MemoryStore {
    type Error = std::convert::Infallible;

    fn load_outgoing_index(&self, channel_id: &ChannelId, epoch: u32) -> Result<Option<u64>, Self::Error> {
        Ok(self.out_indices.get(&(*channel_id, epoch)).copied())
    }

    fn save_outgoing_index(&mut self, channel_id: &ChannelId, epoch: u32, index: u64) -> Result<(), Self::Error> {
        self.out_indices.insert((*channel_id, epoch), index);
        Ok(())
    }

    fn delete_outgoing_index(&mut self, channel_id: &ChannelId, epoch: u32) -> Result<(), Self::Error> {
        self.out_indices.remove(&(*channel_id, epoch));
        Ok(())
    }

    fn iter_outgoing_indices(&self) -> impl Iterator<Item = (ChannelId, u32)> {
        self.out_indices.iter().map(|(k, _)| *k)
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
