use hopr_api::chain::{ChannelId, RedeemableTicket};

use crate::errors::TicketManagerError;

pub trait OutgoingIndexStore {
    type Error: std::error::Error + Send + Sync + 'static;
    fn load_outgoing_index(&self, channel_id: &ChannelId) -> Result<u64, TicketManagerError>;
    fn save_outgoing_index(&self, channel_id: &ChannelId, index: u64) -> Result<(), TicketManagerError>;
}

pub trait TicketStatsStore {
    type Error: std::error::Error + Send + Sync + 'static;
    fn load_stats(&self) -> Result<hopr_api::db::ChannelTicketStatistics, TicketManagerError>;
    fn save_stats(&self, stats: hopr_api::db::ChannelTicketStatistics) -> Result<(), TicketManagerError>;
}

/// Allows loading ticket queues from a storage.
pub trait TicketQueueStore {
    /// Type of queues.
    type Queue: TicketQueue;
    /// Opens or creates a new queue in storage for the given channel.
    fn open_or_create(&self, channel_id: &ChannelId) -> Result<Self::Queue, <Self::Queue as TicketQueue>::Error>;
    /// Iterate over all channel IDs of ticket queues in the storage.
    fn iter_channels(&self) -> impl Iterator<Item = ChannelId>;
}

/// Backend for ticket storage queue.
///
/// The implementations must honor the natural ordering of tickets.
pub trait TicketQueue {
    type Error: std::error::Error + Send + Sync + 'static;
    /// Number of tickets in the queue.
    fn len(&self) -> usize;
    /// Indicates whether the queue is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Add a ticket to the queue.
    fn push(&mut self, ticket: RedeemableTicket) -> Result<(), Self::Error>;
    /// Remove and return the next ticket in-order from the queue.
    fn pop(&mut self) -> Result<Option<RedeemableTicket>, Self::Error>;
    /// Return the next ticket in-order from the queue without removing it.
    fn peek(&self) -> Result<Option<RedeemableTicket>, Self::Error>;
    /// Iterate over all tickets in the queue in **arbitrary** order.
    fn iter_unordered(&self) -> impl Iterator<Item = Result<RedeemableTicket, Self::Error>>;
}
