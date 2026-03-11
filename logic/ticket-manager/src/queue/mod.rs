mod redb;

use hopr_api::chain::{ChannelId, RedeemableTicket};

/// Allows loading ticket queues from a storage.
pub trait TicketQueueStore {
    /// Type of queues.
    type Queue: TicketQueue;
    /// Opens or creates a new queue in storage for the given channel.
    fn open_or_create(&self, channel_id: &ChannelId) -> Result<Self::Queue, <Self::Queue as TicketQueue>::Error>;
    /// Iterate over all channel IDs of ticket queues in the storage.
    fn iter_channels(&self) -> impl Iterator<Item = ChannelId>;
}

/// Simple non-persistent ticket queue store backed by a `dashmap::DashMap` and [`MemoryTicketQueue`].
///
/// Useful for non-persistent and testing scenarios.
#[derive(Clone, Debug, Default)]
pub struct MemoryTicketQueueStore(dashmap::DashMap<ChannelId, MemoryTicketQueue>);

impl TicketQueueStore for MemoryTicketQueueStore {
    type Queue = MemoryTicketQueue;

    fn open_or_create(&self, channel_id: &ChannelId) -> Result<Self::Queue, <Self::Queue as TicketQueue>::Error> {
        Ok(self.0.entry(*channel_id).or_insert_with(MemoryTicketQueue::default).clone())
    }

    fn iter_channels(&self) -> impl Iterator<Item=ChannelId> {
        self.0.iter().map(|e| e.key().clone())
    }
}

/// Backend for ticket storage queue.
///
/// The implementations must honor the natural ordering of tickets.
pub trait TicketQueue {
    type Error: std::error::Error + Send + Sync + 'static;
    /// Number of tickets in the queue.
    fn len(&self) -> usize;
    /// Indicates whether the queue is empty.
    fn is_empty(&self) -> bool { self.len() == 0 }
    /// Add a ticket to the queue.
    fn push(&mut self, ticket: RedeemableTicket) -> Result<(), Self::Error>;
    /// Remove and return the next ticket in-order from the queue.
    fn pop(&mut self) -> Result<Option<RedeemableTicket>, Self::Error>;
    /// Return the next ticket in-order from the queue without removing it.
    fn peek(&self) -> Result<Option<RedeemableTicket>, Self::Error>;
    /// Iterate over all tickets in the queue in **arbitrary** order.
    fn iter_unordered(&self) -> impl Iterator<Item = Result<RedeemableTicket, Self::Error>>;
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
