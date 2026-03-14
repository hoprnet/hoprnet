use hopr_api::{
    chain::{ChannelId, RedeemableTicket},
};
use crate::{
    OutgoingIndexStore,
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
pub struct MemoryTicketQueue(std::collections::BinaryHeap<std::cmp::Reverse<RedeemableTicket>>);

impl TicketQueue for MemoryTicketQueue {
    type Error = std::convert::Infallible;

    fn len(&self) -> usize {
        self.0.len()
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn push(&mut self, ticket: RedeemableTicket) -> Result<(), Self::Error> {
        Ok(self.0.push(std::cmp::Reverse(ticket)))
    }

    fn pop(&mut self) -> Result<Option<RedeemableTicket>, Self::Error> {
        Ok(self.0.pop().map(|ticket| ticket.0))
    }

    fn peek(&self) -> Result<Option<RedeemableTicket>, Self::Error> {
        Ok(self.0.peek().cloned().map(|ticket| ticket.0))
    }

    fn iter_unordered(&self) -> impl Iterator<Item = Result<RedeemableTicket, Self::Error>> {
        self.0.iter().cloned().map(|t| Ok(t.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::tests::*;

    #[test]
    fn memory_queue_maintains_natural_ticket_order() -> anyhow::Result<()> {
        queue_maintains_natural_ticket_order(MemoryTicketQueue::default())
    }

    #[test]
    fn memory_queue_returns_all_tickets() -> anyhow::Result<()> {
        queue_returns_all_tickets(MemoryTicketQueue::default())
    }
    #[test]
    fn memory_queue_is_empty_when_drained() -> anyhow::Result<()> {
        queue_is_empty_when_drained(MemoryTicketQueue::default())
    }

    #[test]
    fn memory_queue_returns_empty_iterator_when_drained() -> anyhow::Result<()> {
        queue_returns_empty_iterator_when_drained(MemoryTicketQueue::default())
    }
    #[test]
    fn memory_queue_returns_correct_total_ticket_value() -> anyhow::Result<()> {
        queue_returns_correct_total_ticket_value(MemoryTicketQueue::default())
    }

    #[test]
    fn memory_queue_returns_correct_total_ticket_value_with_min_index() -> anyhow::Result<()> {
        queue_returns_correct_total_ticket_value_with_min_index(MemoryTicketQueue::default())
    }
}
