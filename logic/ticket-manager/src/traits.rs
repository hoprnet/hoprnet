use hopr_api::chain::{ChannelId, HoprBalance, RedeemableTicket};

/// Allows loading and saving outgoing ticket indices.
pub trait OutgoingIndexStore {
    type Error: std::error::Error + Send + Sync + 'static;
    /// Loads the last used outgoing ticket index for the given channel and epoch.
    ///
    /// If the index is not found, returns `Ok(None)`.
    fn load_outgoing_index(&self, channel_id: &ChannelId, epoch: u32) -> Result<Option<u64>, Self::Error>;
    /// Saves the last used outgoing ticket index for the given channel and epoch.
    fn save_outgoing_index(&mut self, channel_id: &ChannelId, epoch: u32, index: u64) -> Result<(), Self::Error>;
    /// Deletes the outgoing ticket index for the given channel and epoch.
    fn delete_outgoing_index(&mut self, channel_id: &ChannelId, epoch: u32) -> Result<(), Self::Error>;
    /// Iterate over all channel IDs and epochs of outgoing ticket indices in the storage.
    fn iter_outgoing_indices(&self) -> impl Iterator<Item = (ChannelId, u32)>;
}

/// Allows loading ticket queues from a storage.
pub trait TicketQueueStore {
    /// Type of queues.
    type Queue: TicketQueue;
    /// Opens or creates a new queue in storage for the given channel.
    fn open_or_create_queue(
        &mut self,
        channel_id: &ChannelId,
    ) -> Result<Self::Queue, <Self::Queue as TicketQueue>::Error>;
    /// Deletes the queue for the given channel.
    fn delete_queue(&mut self, channel_id: &ChannelId) -> Result<(), <Self::Queue as TicketQueue>::Error>;
    /// Iterate over all channel IDs of ticket queues in the storage.
    fn iter_queues(&self) -> impl Iterator<Item = ChannelId>;
}

/// Backend for ticket storage queue.
///
/// The implementations must honor the natural ordering of tickets.
#[auto_impl::auto_impl(&mut, Box)]
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
    /// Computes the total value of tickets of the given epoch (and optionally minimum given index)
    /// in this queue.
    ///
    /// The default implementation simply [iterates](TicketQueue::iter_unordered) the queue
    /// and sums the total value of matching tickets.
    fn total_value(&self, epoch: u32, min_index: Option<u64>) -> Result<HoprBalance, Self::Error> {
        default_total_value(self, epoch, min_index)
    }
}

pub(crate) fn default_total_value<Q: TicketQueue + ?Sized>(
    queue: &Q,
    epoch: u32,
    min_index: Option<u64>,
) -> Result<HoprBalance, Q::Error> {
    let min_index = min_index.unwrap_or(0);
    Ok(queue
        .iter_unordered()
        .filter_map(|res| {
            res.ok()
                .filter(|t| t.verified_ticket().channel_epoch == epoch && t.verified_ticket().index >= min_index)
                .map(|t| t.verified_ticket().amount)
        })
        .sum())
}
