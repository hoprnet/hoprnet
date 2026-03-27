//! Implements complete logic of ticket management in the HOPR protocol.
//!
//! See the [`HoprTicketManager`] documentation for complete details.

mod backend;
mod errors;
mod traits;
mod utils;

use std::{convert::identity, sync::atomic::AtomicBool};

use futures::{Stream, TryFutureExt};
use hopr_api::{
    chain::{ChainWriteTicketOperations, TicketRedeemError},
    tickets::{ChannelStats, RedemptionResult},
    types::{internal::prelude::*, primitive::prelude::*},
};

#[cfg(feature = "redb")]
pub use crate::backend::{RedbStore, RedbTicketQueue};
use crate::{
    backend::ValueCachedQueue,
    utils::{ChannelTicketQueue, OutgoingIndexCache},
};
pub use crate::{
    backend::{MemoryStore, MemoryTicketQueue},
    errors::TicketManagerError,
    traits::{OutgoingIndexStore, TicketQueue, TicketQueueStore},
};

/// Keeps track of indices for outgoing tickets and (optionally) of incoming redeemable tickets.
///
/// The capabilities of the `HoprTicketManager` are given by the store `S`:
/// - if the store implements [`OutgoingIndexStore`], the outgoing index tracking functions are available.
/// - if the store implements [`TicketQueueStore`], the incoming redeemable ticket management is available.
///
/// It is possible to have both implementations in the same store object. Some store implementations may offer
/// persistence, others may not. The `HoprTicketManager` takes ownership of the store object, and other
/// processes should not attempt to change the store externally. For this reason, stores should not be cloneable.
///
/// The HOPR node type gives typical use-cases of the `HoprTicketManager`:
///
/// - Entry/Exit nodes only need to provide an `OutgoingIndexStore`, since they are dealing with outgoing tickets only.
/// - Relay nodes need to provide a store which implements both `OutgoingIndexStore + TicketQueueStore`, because they
///   need to deal with both outgoing tickets and incoming redeemable tickets.
///
/// To synchronize the on-chain state with the store, it is advised to call
/// [`sync_outgoing_channels`](HoprTicketManager::sync_from_outgoing_channels) (and
/// [`sync_incoming_channels`](HoprTicketManager::sync_from_incoming_channels) if applicable to the chosen store) early
/// after the construction of the manager, to make sure outdated data is discarded early. This is typically done only
/// once after construction and not needed to be done during the life-time of the manager.
///
/// The manager is safe to be shared via an `Arc`. It is typically shared between the packet processing pipelines
/// (outgoing on Entry/Exit nodes, incoming on Relay nodes) and some higher level component
/// that performs redeemable ticket extractions (in the case of a Relay node).
///
/// ### Usage in outgoing packet pipeline
/// The outgoing packet pipeline usually just calls the
/// [`create_multihop_ticket`](HoprTicketManager::next_multihop_ticket) to create a ticket for the next hop on a
/// multi-hop path. To create zero/last-hop tickets, the ticket manager is not needed as these tickets essentially
/// contain bogus data and there's no channel required.
///
/// The outgoing indices are **not** automatically synchronized back to the underlying store for performance reasons.
/// The user is responsible for calling [`save_outgoing_indices`](HoprTicketManager::save_outgoing_indices) to save
/// the outgoing indices to the store.
///
/// This usage is typical for all kinds of nodes (Entry/Relay/Exit).
///
/// ### Usage in incoming packet pipeline
/// The incoming packet pipeline usually just calls the
/// [`insert_incoming_ticket`](HoprTicketManager::insert_incoming_ticket) whenever a new winning, redeemable ticket is
/// received on an incoming channel.
///
/// This usage is typical for Relay nodes only.
///
/// ### Redeemable ticket extraction
/// On Relay nodes, the manager maintains FIFO queues of redeemable tickets per incoming channel.
/// There are two ways to extract tickets from the queue on a Relay:
///
/// 1. redeeming them via [`redeem_stream`](hopr_api::tickets::TicketManagement::redeem_stream)
/// 2. neglecting them via [`neglect_tickets`](hopr_api::tickets::TicketManagement::neglect_tickets)
///
/// Both of these operations extract the tickets in the FIFO order from the queue,
/// making sure that they are always processed in their natural order (by epoch and index).
///
/// Both ticket extraction operations are mutually exclusive and cannot be performed simultaneously.
///
/// ## Locking and lock-contention
/// There are several methods in the `HoprTicketManager` object that are expected to be called
/// in the highly performance-sensitive code, on a per-packet basis.
///
/// ### Outgoing ticket creation
/// The [`create_multihop_ticket`](HoprTicketManager::next_multihop_ticket) method is designed to be
/// high-performance and to be called per each outgoing packet. It is using only atomics to track the outgoing
/// ticket index for a channel. The synchronization to the underlying storage is done on-demand by calling
/// `save_outgoing_indices`, making quick snapshots of the current state of outgoing indices.
/// No significant contention is expected unless `save_outgoing_indices` is called very frequently.`
///
/// ### Incoming winning ticket retrieval
/// The [`insert_incoming_ticket`](HoprTicketManager::insert_incoming_ticket) method is designed to be
/// high-performance and to be called per each incoming packet **after** it has been forwarded to a next hop.
///
/// This operation acquires the write-part of an RW lock (per incoming channel).
/// This may block the hot-path only if one of the following (also write) operations is performed:
///     1. Ticket redemption has just finished in that particular channel, and the redeemed ticket is dropped from the
///     same incoming channel queue.
///     2. Ticket neglection has just finished in that particular channel, and the neglected ticket is dropped from the
///     same incoming channel queue.
///
/// Both of these operations happen rarely, and the write lock is usually held only for a short time. In addition,
/// incoming winning tickets are not supposed to usually happen very often. Therefore, high contention on
/// the write lock is not expected.
///
/// ### Incoming unacknowledged ticket verification
/// The [`unrealized_value`](HoprTicketManager::unrealized_value) method is designed to be high-performance
/// and to be called per each incoming packet **before** it is forwarded to a next hop.
///
/// This operation acquires the read-part of an RW lock (per incoming channel). This may block the hot-path only if
/// one of the following (write) operations is performed at the same moment:
///     1. A new incoming winning ticket is inserted into the same incoming channel queue.
///     2. Ticket redemption has just finished in that particular channel, and the redeemed ticket is dropped from the
///     same incoming channel queue.
///     3. Ticket neglection has just finished in that particular channel, and the neglected ticket is dropped from the
///     same incoming channel queue.
///
/// All 3 of these operations are not expected to happen very often on a single channel; therefore, high contention
/// on the RW lock is not expected.
#[derive(Debug)]
pub struct HoprTicketManager<S, Q> {
    out_idx_tracker: OutgoingIndexCache,
    channel_tickets: dashmap::DashMap<ChannelId, ChannelTicketQueue<ValueCachedQueue<Q>>>,
    store: std::sync::Arc<parking_lot::RwLock<S>>,
}

impl<S, Q> HoprTicketManager<S, Q>
where
    S: OutgoingIndexStore + Send + Sync + 'static,
{
    /// Creates a new ticket manager instance given the desired `store`.
    ///
    /// The instance is supposed to take complete ownership of the `store` object. The store
    /// implementations should not allow
    ///
    /// It is advised to call [`HoprTicketManager::sync_from_outgoing_channels`] and
    /// [`HoprTicketManager::sync_from_incoming_channels`] at least once before the manager
    /// is used any further.
    pub fn new(store: S) -> Result<Self, TicketManagerError> {
        let store = std::sync::Arc::new(parking_lot::RwLock::new(store));
        Ok(Self {
            out_idx_tracker: OutgoingIndexCache::default(),
            channel_tickets: dashmap::DashMap::new(),
            store,
        })
    }

    /// Gets the next usable ticket index for an outgoing ticket in the given channel and epoch.
    ///
    /// This operation is fast and does not immediately put the index into the [`OutgoingIndexStore`].
    ///
    /// The returned value is always guaranteed to be greater or equal to the ticket index on the given `channel`.
    fn next_outgoing_ticket_index(&self, channel: &ChannelEntry) -> u64 {
        let mut next_index = self.out_idx_tracker.next(channel.get_id(), channel.channel_epoch);
        tracing::trace!(%channel, next_index, "next outgoing ticket index");

        let epoch = channel.channel_epoch;

        if next_index < channel.ticket_index {
            // Correct the value in the cache if it was lower than the ticket index on the channel.
            // This sets the value in the cache to the next index after the ticket index on the channel.
            self.out_idx_tracker
                .upsert(channel.get_id(), epoch, channel.ticket_index + 1);
            next_index = channel.ticket_index; // Still, use the channel's ticket index as the next index.
        }

        // If this is the first index in this epoch,
        // remove the previous epoch from the map if any.
        // Epochs always start at 1, ticket indices at 0.
        if next_index == 0 && epoch > 1 && self.out_idx_tracker.remove(channel.get_id(), epoch - 1) {
            tracing::trace!(%channel, prev_epoch = epoch - 1, "removing previous epoch from outgoing index cache");
        }

        next_index
    }

    /// Saves outgoing ticket indices back to the store.
    ///
    /// The operation does nothing if there were no [new tickets created](HoprTicketManager::next_multihop_ticket)
    /// on any tracked channel.
    pub fn save_outgoing_indices(&self) -> Result<(), TicketManagerError> {
        self.out_idx_tracker
            .save(self.store.clone())
            .map_err(TicketManagerError::store)?;
        Ok(())
    }

    /// Synchronizes the outgoing index counters based on the current on-chain channel
    /// state given by `outgoing_channels`.
    ///
    /// Outgoing indices for channels that either are not present in `outgoing_channels` or
    /// not present as opened channels will be removed from the store.
    ///
    /// Outgoing indices for existing open channels in `outgoing_channels` will be either:
    /// - added to the store with their current index and epoch (if not present in the store), or
    /// - updated to the maximum of the two index values (if present in the store)
    ///
    /// It is advised to call this function early after the construction of the `HoprTicketManager`
    /// to ensure pruning of dangling or out-of-date values.
    pub fn sync_from_outgoing_channels(&self, outgoing_channels: &[ChannelEntry]) -> Result<(), TicketManagerError> {
        let outgoing_channels: std::collections::HashSet<_, std::hash::RandomState> =
            outgoing_channels.iter().collect();

        // Purge outdated outgoing indices
        let mut store_read = self.store.upgradable_read();
        let stored_indices = store_read
            .iter_outgoing_indices()
            .map_err(TicketManagerError::store)?
            .collect::<Vec<_>>();
        for (channel_id, epoch) in stored_indices {
            // If any stored outgoing index does not match any currently existing opened channel,
            // remove it from the store
            if !outgoing_channels.iter().any(|channel| {
                channel.status == ChannelStatus::Open
                    && channel.get_id() == &channel_id
                    && channel.channel_epoch == epoch
            }) {
                let mut store_write = parking_lot::RwLockUpgradableReadGuard::upgrade(store_read);
                store_write
                    .delete_outgoing_index(&channel_id, epoch)
                    .map_err(TicketManagerError::store)?;
                store_read = parking_lot::RwLockWriteGuard::downgrade_to_upgradable(store_write);
                tracing::debug!(%channel_id, epoch, "purging outdated outgoing index")
            }
        }

        for channel in outgoing_channels
            .iter()
            .filter(|channel| channel.status == ChannelStatus::Open)
        {
            let id = channel.get_id();

            // Either load a previously stored outgoing index or use the channel's ticket index as a
            // fallback
            let epoch = channel.channel_epoch;
            let index = match store_read.load_outgoing_index(id, epoch) {
                Ok(Some(out_index)) => out_index,
                Ok(None) => 0,
                Err(error) => {
                    tracing::error!(%error, %id, "failed to load outgoing index for channel, falling back to channel ticket index");
                    0
                }
            };

            // Always use the maximum from the stored value and the current ticket index on the channel
            let out_index = index.max(channel.ticket_index);
            self.out_idx_tracker.upsert(id, epoch, out_index);
            tracing::debug!(%id, epoch, out_index, "loaded outgoing ticket index for channel");
        }

        tracing::debug!(
            num_channels = outgoing_channels.len(),
            "synchronized with outgoing channels"
        );
        Ok(())
    }

    /// Creates a ticket for the next hop on a multi-hop path.
    ///
    /// The `current_path_pos` indicates the position of the current hop in the multi-hop path.
    /// It is used to determine the value of the ticket: `price * (current_path_pos - 1) / winning_prob`.
    /// The function does not make sense for `current_path_pos <= 1` and returns an error if such an argument is
    /// provided.
    ///
    /// For last-hop tickets (`current_path_pos` equal to 1), a [zero hop ticket](TicketBuilder::zero_hop) should be
    /// created instead.
    ///
    /// The function will fail for channels that are not opened or do not have enough funds to cover the ticket value.
    /// The ticket index of the returned ticket is guaranteed to be greater or equal to the ticket index on the
    /// given `channel` argument.
    pub fn next_multihop_ticket(
        &self,
        channel: &ChannelEntry,
        current_path_pos: u8,
        winning_prob: WinningProbability,
        ticket_price: HoprBalance,
    ) -> Result<TicketBuilder, TicketManagerError> {
        if current_path_pos <= 1 {
            return Err(TicketManagerError::Other(anyhow::anyhow!(
                "current path position for multihop ticket must be greater than 1"
            )));
        }

        if channel.status != ChannelStatus::Open {
            return Err(TicketManagerError::Other(anyhow::anyhow!(
                "channel must be open to create a multihop ticket"
            )));
        }

        // The next ticket is worth: price * remaining hop count / winning probability
        let amount = HoprBalance::from(
            ticket_price
                .amount()
                .saturating_mul(U256::from(current_path_pos - 1))
                .div_f64(winning_prob.into())
                .expect("winning probability is always less than or equal to 1"),
        );

        if channel.balance.lt(&amount) {
            return Err(TicketManagerError::OutOfFunds(*channel.get_id(), amount));
        }

        let ticket_builder = TicketBuilder::default()
            .counterparty(channel.destination)
            .balance(amount)
            .index(self.next_outgoing_ticket_index(channel))
            .win_prob(winning_prob)
            .channel_epoch(channel.channel_epoch);

        Ok(ticket_builder)
    }
}

struct RedeemState<C, Q> {
    lock: std::sync::Arc<AtomicBool>,
    queue: std::sync::Arc<parking_lot::RwLock<Q>>,
    chain: C,
    min_redeem_value: HoprBalance,
    channel_id: ChannelId,
}

impl<C, Q> Drop for RedeemState<C, Q> {
    fn drop(&mut self) {
        self.lock.store(false, std::sync::atomic::Ordering::Release);
    }
}

impl<S> HoprTicketManager<S, S::Queue>
where
    S: TicketQueueStore + Send + Sync + 'static,
    S::Queue: Send + Sync + 'static,
{
    /// Synchronizes the existing incoming redeemable ticket queues with the state of the
    /// current `incoming_channels`.
    ///
    /// Any incoming ticket queues that correspond to a channel that is no longer open or effectively open (in
    /// `incoming_channels`) will be dropped and the tickets neglected.
    ///
    /// For all opened or effectively opened incoming channels inside `incoming_channels`, either an existing
    /// ticket queue is opened or a new one is created (without any tickets in it).
    ///
    /// All the neglected tickets are returned from the function to make further accounting possible,
    /// but they are no longer redeemable.
    ///
    /// It is advised to call this function early after the construction of the `HoprTicketManager`
    /// to ensure pruning of dangling or out-of-date values.
    pub fn sync_from_incoming_channels(
        &self,
        incoming_channels: &[ChannelEntry],
    ) -> Result<Vec<VerifiedTicket>, TicketManagerError> {
        let incoming_channels: std::collections::HashSet<_, std::hash::RandomState> =
            incoming_channels.iter().collect();

        // Purge outdated incoming channel queues
        let mut store_read = self.store.upgradable_read();
        let stored_queues = store_read
            .iter_queues()
            .map_err(TicketManagerError::store)?
            .collect::<Vec<_>>();
        let mut neglected = Vec::new();
        let now = hopr_platform::time::current_time();
        for channel_id in stored_queues {
            // If any existing redeemable ticket queue does not match any currently existing
            // channel that's either open or its closure period did not yet elapse (i.e., the channel
            // is not closed or not effectively closed), remove the queue from the store.
            if !incoming_channels
                .iter()
                .any(|channel| !channel.closure_time_passed(now) && channel.get_id() == &channel_id)
            {
                let mut store_write = parking_lot::RwLockUpgradableReadGuard::upgrade(store_read);
                neglected.extend(
                    store_write
                        .delete_queue(&channel_id)
                        .map_err(TicketManagerError::store)?,
                );
                tracing::debug!(%channel_id, "purged outdated incoming tickets queue");
                store_read = parking_lot::RwLockWriteGuard::downgrade_to_upgradable(store_write);

                // We cannot account the neglected tickets, because the channel has been closed.
                self.channel_tickets.remove(&channel_id);
            }
        }
        // Create or open ticket queues for all incoming channels that are open or effectively open
        for channel in incoming_channels
            .iter()
            .filter(|channel| !channel.closure_time_passed(now))
        {
            let id = channel.get_id();

            // Either open an existing queue for that channel or create a new one
            let mut store_write = parking_lot::RwLockUpgradableReadGuard::upgrade(store_read);
            let queue = store_write
                .open_or_create_queue(id)
                .map_err(TicketManagerError::store)?;
            store_read = parking_lot::RwLockWriteGuard::downgrade_to_upgradable(store_write);

            // Wrap the queue with a ticket value cache adapter
            let queue = ValueCachedQueue::new(queue).map_err(TicketManagerError::store)?;

            tracing::debug!(%id, num_tickets = queue.len().map_err(TicketManagerError::store)?, "loaded redeemable ticket queue for channel");
            self.channel_tickets.insert(*id, queue.into());
        }

        tracing::debug!(
            num_channels = incoming_channels.len(),
            num_neglected = neglected.len(),
            "synchronized with incoming channels"
        );
        Ok(neglected)
    }

    /// Inserts a new incoming winning redeemable ticket into the ticket manager.
    ///
    /// On success, the method returns all tickets that have been neglected in the ticket queue of this channel,
    /// in case the inserted ticket has a greater channel epoch than the [next extractable](TicketQueue::peek) ticket in
    /// the queue. This situation can happen when unredeemed tickets are left in the queue, while the corresponding
    /// channel restarts its lifecycle and a new winning ticket is received.
    /// Otherwise, the returned vector is empty.
    pub fn insert_incoming_ticket(&self, ticket: RedeemableTicket) -> Result<Vec<VerifiedTicket>, TicketManagerError> {
        // Do not allocate, because neglecting tickets is a rare operation
        let mut neglected_tickets = Vec::with_capacity(0);

        let ticket_id = ticket.ticket_id();
        match self.channel_tickets.entry(ticket_id.id) {
            dashmap::Entry::Occupied(e) => {
                // High contention on this write lock is possible only when massive numbers of winning tickets
                // on the same channel are received, or if tickets on the same channel are being
                // rapidly redeemed or neglected.
                // Such a scenario is likely not realistic.
                let mut queue = e.get().queue.write();

                // If the next ticket ready in this queue is from a previous epoch, we must
                // drain and neglect all the tickets from the queue. The channel has
                // apparently restarted its lifecycle, and all the tickets from previous epochs
                // are unredeemable already
                if let Some(last_ticket) = queue.0.peek().map_err(TicketManagerError::store)? {
                    if last_ticket.verified_ticket().channel_epoch < ticket.verified_ticket().channel_epoch {
                        // Count the neglected value and add it to stats
                        let mut neg = queue.0.drain().map_err(TicketManagerError::store)?;
                        queue.1.neglected_value += neg.iter().map(|t| t.verified_ticket().amount).sum::<HoprBalance>();

                        // Ensures allocation according to the number of drained tickets
                        neglected_tickets.append(&mut neg);
                        tracing::warn!(%ticket_id, num_neglected = neglected_tickets.len(), "winning ticket has neglected unredeemed tickets from previous epochs");
                    } else if last_ticket.verified_ticket().channel_epoch > ticket.verified_ticket().channel_epoch {
                        tracing::warn!(%ticket_id, "tried to insert incoming ticket from an older epoch");

                        queue.1.winning_tickets += 1; // Still count the ticket as winning
                        queue.1.neglected_value += ticket.verified_ticket().amount;
                        neglected_tickets.push(ticket.ticket);
                        return Ok(neglected_tickets);
                    }
                }
                queue.0.push(ticket).map_err(TicketManagerError::store)?;
                queue.1.winning_tickets += 1;

                tracing::debug!(%ticket_id, "winning ticket on channel");
            }
            dashmap::Entry::Vacant(v) => {
                // A hypothetical chance of high contention on this write lock is
                // only possible when massive numbers of winning tickets on new unique channels are received.
                // Such a scenario is likely not realistic.
                let mut store = self.store.write();

                let queue = store
                    .open_or_create_queue(&ticket.ticket_id().id)
                    .map_err(TicketManagerError::store)?;

                // Wrap the queue with a ticket value cache adapter
                let mut queue = ValueCachedQueue::new(queue).map_err(TicketManagerError::store)?;

                // Should not happen: it suggests the queue has been modified outside the manager
                if !queue.is_empty().map_err(TicketManagerError::store)? {
                    return Err(TicketManagerError::Other(anyhow::anyhow!(
                        "fatal error: queue not empty"
                    )));
                }

                queue.push(ticket).map_err(TicketManagerError::store)?;
                v.insert(queue.into()); // The ticket is accounted for in the stats automatically
                tracing::debug!(%ticket_id, "first winning ticket on channel");
            }
        }

        Ok(neglected_tickets)
    }

    /// Returns the total value of unredeemed tickets in the given channel and its latest epoch.
    ///
    /// NOTE: The function is less efficient when the `min_index` is specified, as
    /// a full scan of the queue is required to calculate the unrealized value.
    pub fn unrealized_value(
        &self,
        channel_id: &ChannelId,
        min_index: Option<u64>,
    ) -> Result<Option<HoprBalance>, TicketManagerError> {
        if let Some(ticket_queue) = self.channel_tickets.get(channel_id) {
            // There is low contention on this read lock, because write locks are acquired only
            // when a new winning ticket has been added, redeemed or neglected, all of which are fairly rare operations.
            let queue = ticket_queue.queue.read();

            // Get the epoch of the first extractable ticket in the queue.
            // The ticket insertion takes care that there are no tickets
            // with epochs other than the current epoch.
            if let Some(epoch) = queue
                .0
                .peek()
                .map_err(TicketManagerError::store)?
                .map(|t| t.verified_ticket().channel_epoch)
            {
                Ok(Some(
                    queue
                        .0
                        .total_value(epoch, min_index)
                        .map_err(TicketManagerError::store)?,
                ))
            } else {
                Ok(Some(HoprBalance::zero()))
            }
        } else {
            Ok(None)
        }
    }
}
impl<S> hopr_api::tickets::TicketManagement for HoprTicketManager<S, S::Queue>
where
    S: TicketQueueStore + Send + Sync + 'static,
    S::Queue: Send + Sync + 'static,
{
    type Error = TicketManagerError;

    /// Creates a stream that redeems tickets in-order one by one in the given channel,
    /// using the given [`ChainWriteTicketOperations`] on-chain client
    /// implementation.
    ///
    /// If `min_redeem_value` is given, all the tickets that are lower than the given value are neglected in the
    /// process.
    ///
    /// If there's already an existing redeem stream for the channel, an error is returned without creating a new
    /// stream.
    ///
    /// The stream terminates when there are no more tickets to process in the queue, or an error is encountered.
    fn redeem_stream<C: ChainWriteTicketOperations + Send + Sync + 'static>(
        &self,
        chain: C,
        channel_id: ChannelId,
        min_amount: Option<HoprBalance>,
    ) -> Result<impl Stream<Item = Result<RedemptionResult, Self::Error>> + Send, Self::Error> {
        let initial_state = match self.channel_tickets.get(&channel_id) {
            Some(ticket_queue) => {
                ticket_queue
                    .redeem_lock
                    .compare_exchange(
                        false,
                        true,
                        std::sync::atomic::Ordering::Acquire,
                        std::sync::atomic::Ordering::Relaxed,
                    )
                    .map_err(|_| TicketManagerError::AlreadyRedeeming)?;

                RedeemState {
                    lock: ticket_queue.redeem_lock.clone(),
                    queue: ticket_queue.queue.clone(),
                    min_redeem_value: min_amount.unwrap_or_default(), // default min is 0 wxHOPR
                    chain,
                    channel_id,
                }
            }
            None => return Err(TicketManagerError::ChannelQueueNotFound),
        };

        Ok(futures::stream::try_unfold(initial_state, |state| {
            // Peek here and release the read lock to prevent holding it across an `await`
            let next_ticket = state.queue.read().0.peek();
            async move {
                match next_ticket.map_err(TicketManagerError::store)? {
                    Some(ticket_to_redeem) => {
                        // Attempt to redeem the ticket if it is of sufficient value
                        let redeem_attempt_result =
                            if ticket_to_redeem.verified_ticket().amount >= state.min_redeem_value {
                                match state.chain.redeem_ticket(ticket_to_redeem).and_then(identity).await {
                                    Ok((redeemed_ticket, _)) => Ok(Some(RedemptionResult::Redeemed(redeemed_ticket))),
                                    Err(TicketRedeemError::Rejected(ticket, reason)) => {
                                        Ok(Some(RedemptionResult::RejectedOnChain(ticket, reason)))
                                    }
                                    Err(TicketRedeemError::ProcessingError(_, err)) => {
                                        Err(TicketManagerError::redeem(err))
                                    }
                                }
                            } else {
                                // Tickets with low value are treated as neglected
                                Ok(Some(RedemptionResult::ValueTooLow(ticket_to_redeem.ticket)))
                            };

                        // Once the redemption has been completed, no matter if successful or not,
                        // check if we need to remove the ticket from the redemption queue.
                        if let Ok(Some(redeem_complete_result)) =  &redeem_attempt_result {
                            // In this case, no matter if the ticket has been redeemed,
                            // neglected or rejected, we're still removing it from the queue.
                            // Otherwise, the ticket stays in the queue due to a recoverable error

                            let mut queue_write = state.queue.write();
                            let pop_res = queue_write.0.pop().map_err(TicketManagerError::store)?;

                            // Do accounting of the ticket into the stats
                            match redeem_complete_result {
                                RedemptionResult::Redeemed(ticket) => {
                                    queue_write.1.redeemed_value += ticket.verified_ticket().amount;
                                    tracing::info!(%ticket, "ticket has been redeemed");
                                },
                                RedemptionResult::ValueTooLow(ticket) => {
                                    queue_write.1.neglected_value += ticket.verified_ticket().amount;
                                    tracing::warn!(%ticket, "ticket has been neglected");
                                },
                                RedemptionResult::RejectedOnChain(ticket, reason) => {
                                    queue_write.1.rejected_value += ticket.verified_ticket().amount;
                                    tracing::warn!(%ticket, reason, "ticket has been rejected on-chain");
                                },
                            }

                            // This can only happen if `neglect_tickets` has been called while redeeming,
                            // and it has neglected the ticket during this race-condition.
                            // In this case we only need to correct the neglected value because
                            // the ticket has been actually redeemed/rejected or was accounted
                            // as neglected twice.
                            if pop_res.is_none() {
                                let ticket = redeem_complete_result.as_ref();
                                tracing::warn!(%ticket, "ticket has been neglected from the queue while it actually completed the redemption process");
                                queue_write.1.neglected_value -= ticket.verified_ticket().amount;
                            }
                        }

                        redeem_attempt_result
                    }
                    None => {
                        // No more tickets to redeem in this channel
                        // Keep the queue in even if it is empty. The cleanup is done only on startup.
                        tracing::debug!(channel_id = %state.channel_id, "no more tickets to redeem in channel");
                        Ok(None)
                    }
                }
                .map(|s| s.map(|v| (v, state)))
            }
        }))
    }

    /// Removes all the tickets in the given [`ChannelId`], optionally only up to the given ticket index (inclusive).
    ///
    /// If the `up_to_index` is given and lower than the lowest index of an unredeemed ticket in the queue,
    /// the function does nothing.
    ///
    /// If there's ticket redemption ongoing in the same channel and the neglection intersects with the
    /// redeemed range, the redemption will be cut short, with remaining unredeemed tickets neglected.
    fn neglect_tickets(
        &self,
        channel_id: &ChannelId,
        up_to_index: Option<u64>,
    ) -> Result<Vec<VerifiedTicket>, TicketManagerError> {
        let queue = self
            .channel_tickets
            .get(channel_id)
            .map(|q| {
                if q.redeem_lock.load(std::sync::atomic::Ordering::Relaxed) {
                    tracing::warn!(%channel_id, "neglecting tickets in channel while redeeming is ongoing");
                }
                q.queue.clone()
            })
            .ok_or(TicketManagerError::ChannelQueueNotFound)?;

        let mut neglected_tickets = Vec::new();
        let mut queue_read = queue.upgradable_read();
        let max_index = up_to_index.unwrap_or(TicketBuilder::MAX_TICKET_INDEX);

        while queue_read
            .0
            .peek()
            .map_err(TicketManagerError::store)?
            .filter(|ticket| ticket.verified_ticket().index <= max_index)
            .is_some()
        {
            // Quickly perform pop and downgrade to lock not to block any readers
            let mut queue_write = parking_lot::RwLockUpgradableReadGuard::upgrade(queue_read);
            let maybe_ticket = queue_write.0.pop().map_err(TicketManagerError::store)?;
            queue_write.1.neglected_value += maybe_ticket.map(|t| t.verified_ticket().amount).unwrap_or_default();
            queue_read = parking_lot::RwLockWriteGuard::downgrade_to_upgradable(queue_write);

            neglected_tickets.extend(maybe_ticket.map(|t| t.ticket));
            tracing::debug!(%channel_id, ?maybe_ticket, "neglected ticket in channel");
        }

        // Keep the queue in even if it is empty. The cleanup is done only on startup.
        tracing::debug!(%channel_id, num_tickets = neglected_tickets.len(), "ticket neglection done in channel");
        Ok(neglected_tickets)
    }

    /// Computes [statistics](ChannelStats) for the given `channel` or for all channels if `None` is given.
    ///
    /// If the given `channel` does not exist, it returns zero statistics instead of an error.
    ///
    /// Apart from [`unredeemed_value`](ChannelStats), the statistics are not persistent.
    #[allow(deprecated)] // TODO: remove once blokli#237 is merged
    fn ticket_stats(&self, channel: Option<&ChannelId>) -> Result<ChannelStats, TicketManagerError> {
        self.channel_tickets
            .iter()
            .filter(|e| channel.is_none_or(|c| c == e.key()))
            .try_fold(ChannelStats::default(), |stats, v| {
                let queue = v.queue.read();
                Ok::<_, TicketManagerError>(ChannelStats {
                    winning_tickets: queue.1.winning_tickets + stats.winning_tickets,
                    unredeemed_value: queue
                        .0
                        .peek()
                        .map_err(TicketManagerError::store)?
                        .map(|t| queue.0.total_value(t.verified_ticket().channel_epoch, None))
                        .transpose()
                        .map_err(TicketManagerError::store)?
                        .unwrap_or_default()
                        + stats.unredeemed_value,
                    rejected_value: queue.1.rejected_value + stats.rejected_value,
                    redeemed_value: queue.1.redeemed_value + stats.redeemed_value,
                    neglected_value: queue.1.neglected_value + stats.neglected_value,
                })
            })
    }
}

#[allow(deprecated)] // TODO: remove once blokli#237 is merged
#[cfg(test)]
mod tests {
    use std::ops::Sub;

    use futures::{TryStreamExt, pin_mut};
    use hopr_api::{
        OffchainKeypair,
        tickets::TicketManagement,
        types::crypto::prelude::{ChainKeypair, Keypair},
    };
    use hopr_chain_connector::{
        BlockchainConnectorConfig, HoprBlockchainConnector, InMemoryBackend, PayloadGenerator, SafePayloadGenerator,
        reexports::chain::contract_addresses_for_network,
        testing::{BlokliTestClient, BlokliTestStateBuilder, FullStateEmulator},
    };
    use rand::prelude::SliceRandom;

    use super::*;
    use crate::traits::tests::{generate_owned_tickets, generate_tickets};

    fn create_mgr() -> anyhow::Result<HoprTicketManager<MemoryStore, MemoryTicketQueue>> {
        Ok(HoprTicketManager::new(MemoryStore::default())?)
    }

    #[test]
    fn ticket_manager_non_existing_channel_should_return_empty_stats() -> anyhow::Result<()> {
        let mgr = create_mgr()?;

        assert_eq!(ChannelStats::default(), mgr.ticket_stats(None)?);
        assert_eq!(ChannelStats::default(), mgr.ticket_stats(Some(&ChannelId::default()))?);
        Ok(())
    }

    #[test]
    fn ticket_manager_should_create_multihop_tickets() -> anyhow::Result<()> {
        let mgr = create_mgr()?;

        let src = ChainKeypair::random();
        let dst = ChainKeypair::random();

        let channel = ChannelEntry::builder()
            .between(&src, &dst)
            .amount(10)
            .ticket_index(1)
            .status(ChannelStatus::Open)
            .epoch(1)
            .build()?;

        // Loads index 1 which is the next index for a ticket on this channel
        mgr.sync_from_outgoing_channels(&[channel])?;

        let ticket_1 = mgr
            .next_multihop_ticket(&channel, 2, WinningProbability::ALWAYS, 10.into())?
            .eth_challenge(Default::default())
            .build_signed(&src, &Default::default())?;

        assert_eq!(ticket_1.channel_id(), channel.get_id());
        assert_eq!(channel.ticket_index, ticket_1.verified_ticket().index);
        assert_eq!(channel.channel_epoch, ticket_1.verified_ticket().channel_epoch);

        let ticket_2 = mgr
            .next_multihop_ticket(&channel, 2, WinningProbability::ALWAYS, 10.into())?
            .eth_challenge(Default::default())
            .build_signed(&src, &Default::default())?;

        assert_eq!(ticket_2.channel_id(), channel.get_id());
        assert_eq!(channel.ticket_index + 1, ticket_2.verified_ticket().index);
        assert_eq!(channel.channel_epoch, ticket_2.verified_ticket().channel_epoch);

        Ok(())
    }

    #[test]
    fn ticket_manager_should_update_state_when_winning_tickets_are_inserted() -> anyhow::Result<()> {
        let mgr = create_mgr()?;

        let src = ChainKeypair::random();
        let dst = ChainKeypair::random();

        let channel = ChannelEntry::builder()
            .between(&src, &dst)
            .amount(10)
            .ticket_index(1)
            .status(ChannelStatus::Open)
            .epoch(1)
            .build()?;

        let tickets = generate_owned_tickets(&src, &dst, 2, 1..=1)?;

        mgr.insert_incoming_ticket(tickets[0])?;

        assert_eq!(
            ChannelStats {
                winning_tickets: 1,
                unredeemed_value: tickets[0].verified_ticket().amount,
                rejected_value: HoprBalance::zero(),
                redeemed_value: HoprBalance::zero(),
                neglected_value: HoprBalance::zero(),
            },
            mgr.ticket_stats(Some(&channel.get_id()))?
        );

        mgr.insert_incoming_ticket(tickets[1])?;

        assert_eq!(
            ChannelStats {
                winning_tickets: 2,
                unredeemed_value: tickets[0].verified_ticket().amount + tickets[1].verified_ticket().amount,
                rejected_value: HoprBalance::zero(),
                redeemed_value: HoprBalance::zero(),
                neglected_value: HoprBalance::zero(),
            },
            mgr.ticket_stats(Some(&channel.get_id()))?
        );

        Ok(())
    }

    #[test]
    fn ticket_manager_create_multihop_ticket_should_fail_on_wrong_input() -> anyhow::Result<()> {
        let mgr = create_mgr()?;

        let src = ChainKeypair::random();
        let dst = ChainKeypair::random();

        let mut channel = ChannelEntry::builder()
            .between(&src, &dst)
            .amount(10)
            .ticket_index(1)
            .status(ChannelStatus::Closed)
            .epoch(1)
            .build()?;

        assert!(
            mgr.next_multihop_ticket(&channel, 2, WinningProbability::ALWAYS, 1.into())
                .is_err()
        );

        channel.status =
            ChannelStatus::PendingToClose(std::time::SystemTime::now() - std::time::Duration::from_secs(10));

        assert!(
            mgr.next_multihop_ticket(&channel, 2, WinningProbability::ALWAYS, 1.into())
                .is_err()
        );

        channel.status = ChannelStatus::Open;

        assert!(
            mgr.next_multihop_ticket(&channel, 2, WinningProbability::ALWAYS, 11.into())
                .is_err()
        );

        assert!(
            mgr.next_multihop_ticket(&channel, 1, WinningProbability::ALWAYS, 1.into())
                .is_err()
        );

        Ok(())
    }

    #[test]
    fn ticket_manager_test_next_outgoing_ticket_index() -> anyhow::Result<()> {
        let mgr = create_mgr()?;

        let src = ChainKeypair::random();
        let dst = ChainKeypair::random();

        let mut channel = ChannelEntry::builder()
            .between(&src, &dst)
            .amount(10)
            .ticket_index(0)
            .status(ChannelStatus::Open)
            .epoch(1)
            .build()?;

        assert_eq!(0, mgr.next_outgoing_ticket_index(&channel));

        channel.ticket_index = 10;
        assert_eq!(10, mgr.next_outgoing_ticket_index(&channel));
        assert_eq!(11, mgr.next_outgoing_ticket_index(&channel));

        channel.ticket_index = 100;
        assert_eq!(100, mgr.next_outgoing_ticket_index(&channel));
        assert_eq!(101, mgr.next_outgoing_ticket_index(&channel));

        channel.ticket_index = 50;
        assert_eq!(102, mgr.next_outgoing_ticket_index(&channel));
        assert_eq!(103, mgr.next_outgoing_ticket_index(&channel));

        mgr.save_outgoing_indices()?;
        assert_eq!(Some(104), mgr.store.read().load_outgoing_index(channel.get_id(), 1)?);

        channel.ticket_index = 0;
        channel.channel_epoch = 2;

        assert_eq!(0, mgr.next_outgoing_ticket_index(&channel));
        mgr.save_outgoing_indices()?;

        assert_eq!(None, mgr.store.read().load_outgoing_index(channel.get_id(), 1)?);
        assert_eq!(Some(1), mgr.store.read().load_outgoing_index(channel.get_id(), 2)?);

        assert_eq!(1, mgr.next_outgoing_ticket_index(&channel));

        Ok(())
    }

    #[test]
    fn ticket_manager_should_save_out_indices_to_the_store_on_demand() -> anyhow::Result<()> {
        let mgr = create_mgr()?;

        let src = ChainKeypair::random();
        let dst = ChainKeypair::random();

        let channel = ChannelEntry::builder()
            .between(&src, &dst)
            .amount(10)
            .ticket_index(1)
            .status(ChannelStatus::Open)
            .epoch(1)
            .build()?;

        // Loads index 1 which is the next index for a ticket on this channel
        mgr.sync_from_outgoing_channels(&[channel])?;

        mgr.next_multihop_ticket(&channel, 2, WinningProbability::ALWAYS, 10.into())?;

        // Without saving, the store index should not be present in store
        let saved_index = mgr.store.read().load_outgoing_index(channel.get_id(), 1)?;
        assert_eq!(None, saved_index);

        mgr.next_multihop_ticket(&channel, 2, WinningProbability::ALWAYS, 10.into())?;

        mgr.save_outgoing_indices()?;
        let saved_index = mgr.store.read().load_outgoing_index(channel.get_id(), 1)?;
        assert_eq!(Some(3), saved_index);

        mgr.next_multihop_ticket(&channel, 2, WinningProbability::ALWAYS, 10.into())?;

        let saved_index = mgr.store.read().load_outgoing_index(channel.get_id(), 1)?;
        assert_eq!(Some(3), saved_index);

        mgr.save_outgoing_indices()?;
        let saved_index = mgr.store.read().load_outgoing_index(channel.get_id(), 1)?;
        assert_eq!(Some(4), saved_index);

        Ok(())
    }

    #[test]
    fn ticket_manager_should_sync_out_indices_from_chain_state() -> anyhow::Result<()> {
        let mgr = create_mgr()?;

        let src = ChainKeypair::random();
        let dst = ChainKeypair::random();

        let channel = ChannelEntry::builder()
            .between(&src, &dst)
            .amount(10)
            .ticket_index(1)
            .status(ChannelStatus::Open)
            .epoch(1)
            .build()?;

        mgr.sync_from_outgoing_channels(&[channel])?;
        mgr.save_outgoing_indices()?;

        let saved_index = mgr.store.read().load_outgoing_index(channel.get_id(), 1)?;
        assert_eq!(Some(1), saved_index);

        Ok(())
    }

    #[test_log::test]
    fn ticket_manager_should_sync_out_indices_should_remove_indices_for_non_opened_outgoing_channels()
    -> anyhow::Result<()> {
        let mgr = create_mgr()?;

        let src = ChainKeypair::random();
        let dst = ChainKeypair::random();

        let mut channel_1 = ChannelEntry::builder()
            .between(&src, &dst)
            .amount(10)
            .ticket_index(0)
            .status(ChannelStatus::Open)
            .epoch(1)
            .build()?;

        let mut channel_2 = ChannelEntry::builder()
            .between(&dst, &src)
            .amount(10)
            .ticket_index(0)
            .status(ChannelStatus::Open)
            .epoch(1)
            .build()?;

        let ticket_1 = mgr
            .next_multihop_ticket(&channel_1, 2, WinningProbability::ALWAYS, 10.into())?
            .eth_challenge(Default::default())
            .build()?;
        let ticket_2 = mgr
            .next_multihop_ticket(&channel_2, 2, WinningProbability::ALWAYS, 10.into())?
            .eth_challenge(Default::default())
            .build()?;
        assert_eq!(0, ticket_1.index);
        assert_eq!(0, ticket_2.index);

        mgr.save_outgoing_indices()?;

        assert_eq!(Some(1), mgr.store.read().load_outgoing_index(channel_1.get_id(), 1)?);
        assert_eq!(Some(1), mgr.store.read().load_outgoing_index(channel_2.get_id(), 1)?);

        channel_1.status = ChannelStatus::Closed;
        channel_2.status =
            ChannelStatus::PendingToClose(std::time::SystemTime::now() - std::time::Duration::from_mins(10));

        mgr.sync_from_outgoing_channels(&[channel_1, channel_2])?;

        assert_eq!(None, mgr.store.read().load_outgoing_index(channel_1.get_id(), 1)?);
        assert_eq!(None, mgr.store.read().load_outgoing_index(channel_2.get_id(), 1)?);

        Ok(())
    }

    #[test]
    fn ticket_manager_should_sync_incoming_channels_from_chain_state() -> anyhow::Result<()> {
        let mgr = create_mgr()?;

        let src = ChainKeypair::random();
        let dst = ChainKeypair::random();

        let channel = ChannelEntry::builder()
            .between(&src, &dst)
            .amount(10)
            .ticket_index(1)
            .status(ChannelStatus::Open)
            .epoch(1)
            .build()?;

        let neglected = mgr.sync_from_incoming_channels(&[channel])?;
        assert!(neglected.is_empty());

        let queues = mgr.store.read().iter_queues()?.collect::<Vec<_>>();
        assert_eq!(vec![*channel.get_id()], queues);

        Ok(())
    }

    #[test]
    fn ticket_manager_should_neglect_tickets_from_closed_channels_on_sync() -> anyhow::Result<()> {
        let mgr = create_mgr()?;

        let tickets = generate_tickets()?;
        let neglected = mgr.insert_incoming_ticket(tickets[0])?;
        assert!(neglected.is_empty());

        let channel = ChannelEntry::builder()
            .between(
                *tickets[0].ticket.verified_issuer(),
                tickets[0].verified_ticket().counterparty,
            )
            .amount(10)
            .ticket_index(1)
            .status(ChannelStatus::Closed)
            .epoch(1)
            .build()?;

        let neglected = mgr.sync_from_incoming_channels(&[channel])?;
        assert_eq!(1, neglected.len());
        assert_eq!(tickets[0].ticket, neglected[0]);

        Ok(())
    }

    #[test]
    fn ticket_manager_should_neglect_tickets_from_effectively_closed_channels_on_sync() -> anyhow::Result<()> {
        let mgr = create_mgr()?;

        let tickets = generate_tickets()?;
        let neglected = mgr.insert_incoming_ticket(tickets[0])?;
        assert!(neglected.is_empty());

        let channel = ChannelEntry::builder()
            .between(
                *tickets[0].ticket.verified_issuer(),
                tickets[0].verified_ticket().counterparty,
            )
            .amount(10)
            .ticket_index(1)
            .status(ChannelStatus::PendingToClose(
                std::time::SystemTime::now().sub(std::time::Duration::from_mins(10)),
            ))
            .epoch(1)
            .build()?;

        let neglected = mgr.sync_from_incoming_channels(&[channel])?;
        assert_eq!(1, neglected.len());
        assert_eq!(tickets[0].ticket, neglected[0]);

        Ok(())
    }

    #[test]
    fn ticket_manager_should_neglect_tickets_from_non_existent_channels_on_sync() -> anyhow::Result<()> {
        let mgr = create_mgr()?;

        let tickets = generate_tickets()?;

        let neglected = mgr.insert_incoming_ticket(tickets[0])?;
        assert!(neglected.is_empty());

        let neglected = mgr.sync_from_incoming_channels(&[])?;
        assert_eq!(1, neglected.len());
        assert_eq!(tickets[0].ticket, neglected[0]);

        Ok(())
    }

    #[test]
    fn ticket_manager_should_neglect_tickets_on_demand() -> anyhow::Result<()> {
        let mgr = create_mgr()?;

        let tickets = generate_tickets()?;
        let epoch = tickets[0].ticket_id().epoch;

        let tickets = tickets
            .into_iter()
            .filter(|t| t.verified_ticket().channel_epoch == epoch)
            .collect::<Vec<_>>();

        let channel = ChannelEntry::builder()
            .between(
                *tickets[0].ticket.verified_issuer(),
                tickets[0].verified_ticket().counterparty,
            )
            .amount(10)
            .ticket_index(tickets.len() as u64)
            .status(ChannelStatus::Open)
            .epoch(1)
            .build()?;

        for ticket in tickets.iter() {
            let neglected = mgr.insert_incoming_ticket(*ticket)?;
            assert!(neglected.is_empty());
        }

        let neglected = mgr.sync_from_incoming_channels(&[channel])?;
        assert!(neglected.is_empty());

        let unrealized_value = mgr
            .unrealized_value(channel.get_id(), None)?
            .ok_or(anyhow::anyhow!("must have unrealized value"))?;
        assert_eq!(
            tickets.iter().map(|t| t.verified_ticket().amount).sum::<HoprBalance>(),
            unrealized_value
        );

        let neglected = mgr.neglect_tickets(&channel.get_id(), None)?;
        assert_eq!(tickets.iter().map(|t| t.ticket).collect::<Vec<_>>(), neglected);

        let unrealized_value_after = mgr
            .unrealized_value(channel.get_id(), None)?
            .ok_or(anyhow::anyhow!("must have unrealized value"))?;
        assert_eq!(
            unrealized_value_after,
            unrealized_value
                - neglected
                    .iter()
                    .map(|t| t.verified_ticket().amount)
                    .sum::<HoprBalance>()
        );

        assert_eq!(
            ChannelStats {
                winning_tickets: tickets.len() as u128,
                unredeemed_value: unrealized_value_after,
                rejected_value: HoprBalance::zero(),
                redeemed_value: HoprBalance::zero(),
                neglected_value: neglected.iter().map(|t| t.verified_ticket().amount).sum(),
            },
            mgr.ticket_stats(Some(&channel.get_id()))?
        );

        Ok(())
    }

    #[test]
    fn ticket_manager_should_neglect_tickets_on_demand_with_upper_limit_on_index() -> anyhow::Result<()> {
        let mgr = create_mgr()?;

        let tickets = generate_tickets()?;
        let epoch = tickets[0].ticket_id().epoch;

        let tickets = tickets
            .into_iter()
            .filter(|t| t.verified_ticket().channel_epoch == epoch)
            .collect::<Vec<_>>();

        let channel = ChannelEntry::builder()
            .between(
                *tickets[0].ticket.verified_issuer(),
                tickets[0].verified_ticket().counterparty,
            )
            .amount(10)
            .ticket_index(tickets.len() as u64)
            .status(ChannelStatus::Open)
            .epoch(1)
            .build()?;

        for ticket in tickets.iter() {
            let neglected = mgr.insert_incoming_ticket(*ticket)?;
            assert!(neglected.is_empty());
        }

        let neglected = mgr.sync_from_incoming_channels(&[channel])?;
        assert!(neglected.is_empty());

        let unrealized_value = mgr
            .unrealized_value(channel.get_id(), None)?
            .ok_or(anyhow::anyhow!("must have unrealized value"))?;
        assert_eq!(
            tickets.iter().map(|t| t.verified_ticket().amount).sum::<HoprBalance>(),
            unrealized_value
        );

        let neglected = mgr.neglect_tickets(&channel.get_id(), Some(3))?;
        assert_eq!(
            tickets
                .iter()
                .filter(|t| t.verified_ticket().index <= 3)
                .map(|t| t.ticket)
                .collect::<Vec<_>>(),
            neglected
        );

        let unrealized_value_after = mgr
            .unrealized_value(channel.get_id(), None)?
            .ok_or(anyhow::anyhow!("must have unrealized value"))?;
        assert_eq!(
            unrealized_value_after,
            unrealized_value
                - neglected
                    .iter()
                    .map(|t| t.verified_ticket().amount)
                    .sum::<HoprBalance>()
        );

        assert_eq!(
            ChannelStats {
                winning_tickets: tickets.len() as u128,
                unredeemed_value: unrealized_value_after,
                rejected_value: HoprBalance::zero(),
                redeemed_value: HoprBalance::zero(),
                neglected_value: neglected.iter().map(|t| t.verified_ticket().amount).sum(),
            },
            mgr.ticket_stats(Some(&channel.get_id()))?
        );

        Ok(())
    }

    #[test]
    fn ticket_manager_unrealized_value_should_increase_when_tickets_are_added() -> anyhow::Result<()> {
        let mgr = create_mgr()?;

        let mut tickets = generate_tickets()?;
        let channel_id = tickets[0].ticket_id().id;
        let epoch = tickets[0].ticket_id().epoch;
        tickets.retain(|ticket| ticket.verified_ticket().channel_epoch == epoch);

        assert!(!tickets.is_empty());

        assert!(matches!(mgr.unrealized_value(&channel_id, None), Ok(None)));

        let mut last_unrealized_value = HoprBalance::zero();
        assert_eq!(HoprBalance::zero(), last_unrealized_value);

        for ticket in tickets.iter() {
            let neglected = mgr.insert_incoming_ticket(*ticket)?;
            assert!(neglected.is_empty());

            let new_unrealized_value = mgr
                .unrealized_value(&channel_id, None)?
                .ok_or(anyhow::anyhow!("must have unrealized value"))?;
            assert_eq!(
                new_unrealized_value - last_unrealized_value,
                ticket.verified_ticket().amount
            );

            last_unrealized_value = new_unrealized_value;
        }

        let expected_unrealized_value: HoprBalance = tickets.iter().map(|ticket| ticket.verified_ticket().amount).sum();
        assert_eq!(expected_unrealized_value, last_unrealized_value);

        assert_eq!(
            ChannelStats {
                winning_tickets: tickets.len() as u128,
                unredeemed_value: expected_unrealized_value,
                rejected_value: HoprBalance::zero(),
                redeemed_value: HoprBalance::zero(),
                neglected_value: HoprBalance::zero(),
            },
            mgr.ticket_stats(Some(&tickets[0].ticket.channel_id()))?
        );

        Ok(())
    }

    #[test]
    fn ticket_manager_inserted_ticket_with_older_epoch_should_be_neglected() -> anyhow::Result<()> {
        let mgr = create_mgr()?;

        let tickets = generate_tickets()?;
        assert!(!tickets.is_empty());
        let channel_id = tickets[0].ticket_id().id;

        let tickets_from_epoch_1 = tickets
            .iter()
            .filter(|ticket| ticket.verified_ticket().channel_epoch == 1)
            .cloned()
            .collect::<Vec<_>>();
        assert!(!tickets_from_epoch_1.is_empty());

        let tickets_from_epoch_2 = tickets
            .iter()
            .filter(|ticket| ticket.verified_ticket().channel_epoch == 2)
            .cloned()
            .collect::<Vec<_>>();
        assert!(!tickets_from_epoch_2.is_empty());

        for new_ticket in &tickets_from_epoch_2 {
            let neglected = mgr.insert_incoming_ticket(*new_ticket)?;
            assert!(neglected.is_empty());
        }

        for old_ticket in &tickets_from_epoch_1 {
            let neglected = mgr.insert_incoming_ticket(*old_ticket)?;
            assert_eq!(vec![old_ticket.ticket], neglected);
        }

        let stats = mgr.ticket_stats(Some(&channel_id))?;

        assert_eq!(
            (tickets_from_epoch_1.len() + tickets_from_epoch_2.len()) as u128,
            stats.winning_tickets
        );
        assert_eq!(
            tickets_from_epoch_2
                .iter()
                .map(|t| t.verified_ticket().amount)
                .sum::<HoprBalance>(),
            stats.unredeemed_value
        );
        assert_eq!(HoprBalance::zero(), stats.rejected_value);
        assert_eq!(HoprBalance::zero(), stats.redeemed_value);

        Ok(())
    }

    #[test]
    fn ticket_manager_ticket_insertion_should_neglect_tickets_from_previous_epochs() -> anyhow::Result<()> {
        let mgr = create_mgr()?;

        let tickets = generate_tickets()?;
        assert!(!tickets.is_empty());
        let channel_id = tickets[0].ticket_id().id;

        let tickets_from_epoch_1 = tickets
            .iter()
            .filter(|ticket| ticket.verified_ticket().channel_epoch == 1)
            .cloned()
            .collect::<Vec<_>>();
        assert!(!tickets_from_epoch_1.is_empty());

        let tickets_from_epoch_2 = tickets
            .iter()
            .filter(|ticket| ticket.verified_ticket().channel_epoch == 2)
            .cloned()
            .collect::<Vec<_>>();
        assert!(!tickets_from_epoch_2.is_empty());

        assert!(matches!(mgr.unrealized_value(&channel_id, None), Ok(None)));

        for ticket in tickets_from_epoch_1.iter() {
            let neglected = mgr.insert_incoming_ticket(*ticket)?;
            assert!(neglected.is_empty());
        }

        let new_unrealized_value = mgr
            .unrealized_value(&channel_id, None)?
            .ok_or(anyhow::anyhow!("must have unrealized value"))?;
        assert_eq!(
            new_unrealized_value,
            tickets_from_epoch_1
                .iter()
                .map(|ticket| ticket.verified_ticket().amount)
                .sum()
        );

        let neglected = mgr.insert_incoming_ticket(tickets_from_epoch_2[0].clone())?;
        assert_eq!(
            tickets_from_epoch_1.iter().map(|t| t.ticket).collect::<Vec<_>>(),
            neglected
        );

        // There's now only 1 ticket from epoch 2
        let new_unrealized_value = mgr
            .unrealized_value(&channel_id, None)?
            .ok_or(anyhow::anyhow!("must have unrealized value"))?;
        assert_eq!(tickets_from_epoch_2[0].verified_ticket().amount, new_unrealized_value);

        assert_eq!(
            ChannelStats {
                winning_tickets: tickets_from_epoch_1.len() as u128 + 1,
                unredeemed_value: new_unrealized_value,
                rejected_value: HoprBalance::zero(),
                redeemed_value: HoprBalance::zero(),
                neglected_value: neglected.iter().map(|t| t.verified_ticket().amount).sum(),
            },
            mgr.ticket_stats(Some(&channel_id))?
        );

        let queue_tickets = mgr
            .store
            .write()
            .open_or_create_queue(&channel_id)?
            .iter_unordered()?
            .collect::<Result<Vec<_>, _>>()?;
        assert_eq!(1, queue_tickets.len());
        assert_eq!(
            tickets_from_epoch_2[0].verified_ticket(),
            queue_tickets[0].verified_ticket()
        );

        Ok(())
    }

    pub type TestConnector = HoprBlockchainConnector<
        BlokliTestClient<FullStateEmulator>,
        InMemoryBackend,
        SafePayloadGenerator,
        <SafePayloadGenerator as PayloadGenerator>::TxRequest,
    >;

    async fn create_test_connector(
        private_key: &ChainKeypair,
        channel: &ChannelEntry,
        tx_sim_delay: Option<std::time::Duration>,
    ) -> anyhow::Result<TestConnector> {
        let module_addr: [u8; 20] = [1; 20];
        // We need to be channel destination because we'll be redeeming tickets
        assert_eq!(private_key.public().to_address(), channel.destination);

        let blokli_client = BlokliTestStateBuilder::default()
            .with_balances([(private_key.public().to_address(), XDaiBalance::new_base(1))])
            .with_accounts([
                (
                    AccountEntry {
                        public_key: *OffchainKeypair::random().public(),
                        chain_addr: private_key.public().to_address(),
                        entry_type: AccountType::NotAnnounced,
                        safe_address: None,
                        key_id: 1.into(),
                    },
                    HoprBalance::new_base(1000),
                    XDaiBalance::new_base(1),
                ),
                (
                    AccountEntry {
                        public_key: *OffchainKeypair::random().public(),
                        chain_addr: channel.source,
                        entry_type: AccountType::NotAnnounced,
                        safe_address: None,
                        key_id: 2.into(),
                    },
                    HoprBalance::new_base(1000),
                    XDaiBalance::new_base(1),
                ),
            ])
            .with_channels([*channel])
            .with_hopr_network_chain_info("rotsee")
            .build_dynamic_client(module_addr.into())
            .with_tx_simulation_delay(tx_sim_delay.unwrap_or(std::time::Duration::from_millis(500)));

        let mut connector = TestConnector::new(
            private_key.clone(),
            BlockchainConnectorConfig::default(),
            blokli_client,
            InMemoryBackend::default(),
            SafePayloadGenerator::new(
                &private_key,
                contract_addresses_for_network("rotsee").unwrap().1,
                module_addr.into(),
            ),
        );
        connector.connect().await?;

        Ok(connector)
    }

    #[test_log::test(tokio::test)]
    async fn ticket_manager_should_redeem_tickets_on_demand() -> anyhow::Result<()> {
        let mgr = create_mgr()?;

        let src = ChainKeypair::random();
        let dst = ChainKeypair::random();

        let channel = ChannelEntry::builder()
            .between(&src, &dst)
            .amount(10_000_000_000_u64)
            .ticket_index(0)
            .status(ChannelStatus::Open)
            .epoch(1)
            .build()?;

        let mut tickets = generate_owned_tickets(&src, &dst, 3, 1..=1)?;
        tickets.shuffle(&mut rand::rng());

        for ticket in tickets.iter() {
            assert!(mgr.insert_incoming_ticket(*ticket)?.is_empty());
        }

        tickets.sort();

        let mut unrealized_value = mgr
            .unrealized_value(channel.get_id(), None)?
            .ok_or(anyhow::anyhow!("must have unrealized value"))?;
        assert_eq!(
            tickets.iter().map(|t| t.verified_ticket().amount).sum::<HoprBalance>(),
            unrealized_value
        );

        let connector = create_test_connector(&dst, &channel, None).await?;

        let stream = mgr.redeem_stream(connector, *channel.get_id(), None)?;

        pin_mut!(stream);

        assert_eq!(
            Some(RedemptionResult::Redeemed(tickets[0].ticket)),
            stream.try_next().await?
        );
        assert_eq!(
            mgr.unrealized_value(channel.get_id(), None)?
                .ok_or(anyhow::anyhow!("must have unrealized value"))?,
            unrealized_value - tickets[0].verified_ticket().amount
        );
        unrealized_value = mgr
            .unrealized_value(channel.get_id(), None)?
            .ok_or(anyhow::anyhow!("must have unrealized value"))?;

        assert_eq!(
            Some(RedemptionResult::Redeemed(tickets[1].ticket)),
            stream.try_next().await?
        );
        assert_eq!(
            mgr.unrealized_value(channel.get_id(), None)?
                .ok_or(anyhow::anyhow!("must have unrealized value"))?,
            unrealized_value - tickets[1].verified_ticket().amount
        );
        unrealized_value = mgr
            .unrealized_value(channel.get_id(), None)?
            .ok_or(anyhow::anyhow!("must have unrealized value"))?;

        assert_eq!(
            Some(RedemptionResult::Redeemed(tickets[2].ticket)),
            stream.try_next().await?
        );
        assert_eq!(
            mgr.unrealized_value(channel.get_id(), None)?
                .ok_or(anyhow::anyhow!("must have unrealized value"))?,
            unrealized_value - tickets[2].verified_ticket().amount
        );

        assert_eq!(None, stream.try_next().await?);

        Ok(())
    }

    #[tokio::test]
    async fn ticket_manager_should_not_allow_concurrent_redemptions_on_the_same_channel() -> anyhow::Result<()> {
        let mgr = create_mgr()?;

        let src = ChainKeypair::random();
        let dst = ChainKeypair::random();

        let channel = ChannelEntry::builder()
            .between(&src, &dst)
            .amount(10_000_000_000_u64)
            .ticket_index(0)
            .status(ChannelStatus::Open)
            .epoch(1)
            .build()?;

        let mut tickets = generate_owned_tickets(&src, &dst, 3, 1..=1)?;
        tickets.shuffle(&mut rand::rng());

        for ticket in tickets.iter() {
            assert!(mgr.insert_incoming_ticket(*ticket)?.is_empty());
        }

        let connector = std::sync::Arc::new(create_test_connector(&dst, &channel, None).await?);

        let stream = mgr.redeem_stream(connector.clone(), *channel.get_id(), None)?;

        assert!(mgr.redeem_stream(connector.clone(), *channel.get_id(), None).is_err());

        drop(stream);

        assert!(mgr.redeem_stream(connector.clone(), *channel.get_id(), None).is_ok());

        Ok(())
    }

    #[tokio::test]
    async fn ticket_manager_ticket_neglection_should_cut_ongoing_redemption_short() -> anyhow::Result<()> {
        let mgr = create_mgr()?;

        let src = ChainKeypair::random();
        let dst = ChainKeypair::random();

        let channel = ChannelEntry::builder()
            .between(&src, &dst)
            .amount(10_000_000_000_u64)
            .ticket_index(0)
            .status(ChannelStatus::Open)
            .epoch(1)
            .build()?;

        let mut tickets = generate_owned_tickets(&src, &dst, 3, 1..=1)?;
        tickets.shuffle(&mut rand::rng());

        for ticket in tickets.iter() {
            assert!(mgr.insert_incoming_ticket(*ticket)?.is_empty());
        }

        tickets.sort();

        let unrealized_value = mgr
            .unrealized_value(channel.get_id(), None)?
            .ok_or(anyhow::anyhow!("must have unrealized value"))?;
        assert_eq!(
            tickets.iter().map(|t| t.verified_ticket().amount).sum::<HoprBalance>(),
            unrealized_value
        );

        let connector = std::sync::Arc::new(create_test_connector(&dst, &channel, None).await?);

        let stream = mgr.redeem_stream(connector.clone(), *channel.get_id(), None)?;
        pin_mut!(stream);

        assert_eq!(
            Some(RedemptionResult::Redeemed(tickets[0].ticket)),
            stream.try_next().await?
        );
        assert_eq!(
            mgr.unrealized_value(channel.get_id(), None)?
                .ok_or(anyhow::anyhow!("must have unrealized value"))?,
            unrealized_value - tickets[0].verified_ticket().amount
        );

        let neglected = mgr.neglect_tickets(&channel.get_id(), None)?;
        assert_eq!(
            tickets.into_iter().skip(1).map(|t| t.ticket).collect::<Vec<_>>(),
            neglected
        );
        assert_eq!(
            HoprBalance::zero(),
            mgr.unrealized_value(channel.get_id(), None)?
                .ok_or(anyhow::anyhow!("must have unrealized value"))?
        );

        assert_eq!(None, stream.try_next().await?);

        Ok(())
    }

    #[tokio::test]
    async fn ticket_manager_partial_ticket_neglection_should_cut_ongoing_redemption_short() -> anyhow::Result<()> {
        let mgr = create_mgr()?;

        let src = ChainKeypair::random();
        let dst = ChainKeypair::random();

        let channel = ChannelEntry::builder()
            .between(&src, &dst)
            .amount(10_000_000_000_u64)
            .ticket_index(0)
            .status(ChannelStatus::Open)
            .epoch(1)
            .build()?;

        let mut tickets = generate_owned_tickets(&src, &dst, 5, 1..=1)?;
        tickets.shuffle(&mut rand::rng());

        for ticket in tickets.iter() {
            assert!(mgr.insert_incoming_ticket(*ticket)?.is_empty());
        }

        tickets.sort();

        let mut unrealized_value = mgr
            .unrealized_value(channel.get_id(), None)?
            .ok_or(anyhow::anyhow!("must have unrealized value"))?;
        assert_eq!(
            tickets.iter().map(|t| t.verified_ticket().amount).sum::<HoprBalance>(),
            unrealized_value
        );

        let connector = std::sync::Arc::new(create_test_connector(&dst, &channel, None).await?);

        let stream = mgr.redeem_stream(connector.clone(), *channel.get_id(), None)?;
        pin_mut!(stream);

        // Ticket with index 0 gets redeemed
        assert_eq!(
            Some(RedemptionResult::Redeemed(tickets[0].ticket)),
            stream.try_next().await?
        );
        assert_eq!(
            mgr.unrealized_value(channel.get_id(), None)?
                .ok_or(anyhow::anyhow!("must have unrealized value"))?,
            unrealized_value - tickets[0].verified_ticket().amount
        );
        unrealized_value = mgr
            .unrealized_value(channel.get_id(), None)?
            .ok_or(anyhow::anyhow!("must have unrealized value"))?;

        // Tickets with index 1,2 and 3 get neglected
        let neglected = mgr.neglect_tickets(&channel.get_id(), Some(tickets[3].verified_ticket().index))?;
        assert_eq!(
            tickets.iter().skip(1).take(3).map(|t| t.ticket).collect::<Vec<_>>(),
            neglected
        );
        assert_eq!(
            unrealized_value
                - neglected
                    .into_iter()
                    .map(|t| t.verified_ticket().amount)
                    .sum::<HoprBalance>(),
            mgr.unrealized_value(channel.get_id(), None)?
                .ok_or(anyhow::anyhow!("must have unrealized value"))?
        );

        // The last ticket with index 4 gets redeemed again
        assert_eq!(
            Some(RedemptionResult::Redeemed(tickets[4].ticket)),
            stream.try_next().await?
        );

        assert_eq!(
            HoprBalance::zero(),
            mgr.unrealized_value(channel.get_id(), None)?
                .ok_or(anyhow::anyhow!("must have unrealized value"))?
        );

        assert_eq!(None, stream.try_next().await?);

        Ok(())
    }

    #[tokio::test]
    async fn ticket_manager_ticket_neglection_during_on_chain_redemption_should_be_detected() -> anyhow::Result<()> {
        let mgr = std::sync::Arc::new(create_mgr()?);

        let src = ChainKeypair::random();
        let dst = ChainKeypair::random();

        let channel = ChannelEntry::builder()
            .between(&src, &dst)
            .amount(10_000_000_000_u64)
            .ticket_index(0)
            .status(ChannelStatus::Open)
            .epoch(1)
            .build()?;

        let mut tickets = generate_owned_tickets(&src, &dst, 5, 1..=1)?;
        tickets.shuffle(&mut rand::rng());

        for ticket in tickets.iter() {
            assert!(mgr.insert_incoming_ticket(*ticket)?.is_empty());
        }

        tickets.sort();

        let connector =
            std::sync::Arc::new(create_test_connector(&dst, &channel, Some(std::time::Duration::from_secs(2))).await?);

        let mgr_clone = mgr.clone();
        let jh = tokio::task::spawn(async move {
            let stream = mgr_clone.redeem_stream(connector.clone(), *channel.get_id(), None)?;
            pin_mut!(stream);
            stream.try_next().await
        });
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // All the tickets will appear as neglected
        let neglected = mgr.neglect_tickets(&channel.get_id(), None)?;
        assert_eq!(neglected, tickets.iter().map(|t| t.ticket).collect::<Vec<_>>());

        assert_eq!(
            ChannelStats {
                winning_tickets: tickets.len() as u128,
                unredeemed_value: HoprBalance::zero(),
                rejected_value: HoprBalance::zero(),
                redeemed_value: HoprBalance::zero(),
                neglected_value: neglected.iter().map(|t| t.verified_ticket().amount).sum(),
            },
            mgr.ticket_stats(Some(&channel.get_id()))?
        );

        // Once redemption completes we should see the tickets as redeemed
        let res = jh.await??;
        assert_eq!(Some(RedemptionResult::Redeemed(tickets[0].ticket)), res);

        assert_eq!(
            ChannelStats {
                winning_tickets: tickets.len() as u128,
                unredeemed_value: HoprBalance::zero(),
                rejected_value: HoprBalance::zero(),
                redeemed_value: tickets[0].verified_ticket().amount,
                neglected_value: neglected
                    .iter()
                    .map(|t| t.verified_ticket().amount)
                    .sum::<HoprBalance>()
                    - tickets[0].verified_ticket().amount,
            },
            mgr.ticket_stats(Some(&channel.get_id()))?
        );

        Ok(())
    }

    #[tokio::test]
    async fn ticket_manager_ticket_redemption_should_skip_low_value_tickets() -> anyhow::Result<()> {
        let mgr = create_mgr()?;

        let src = ChainKeypair::random();
        let dst = ChainKeypair::random();

        let channel = ChannelEntry::builder()
            .between(&src, &dst)
            .amount(10_000_000_000_u64)
            .ticket_index(0)
            .status(ChannelStatus::Open)
            .epoch(1)
            .build()?;

        let mut tickets = generate_owned_tickets(&src, &dst, 5, 1..=1)?;
        tickets.shuffle(&mut rand::rng());

        for ticket in tickets.iter() {
            assert!(mgr.insert_incoming_ticket(*ticket)?.is_empty());
        }

        tickets.sort();

        let unrealized_value = mgr
            .unrealized_value(channel.get_id(), None)?
            .ok_or(anyhow::anyhow!("must have unrealized value"))?;
        assert_eq!(
            tickets.iter().map(|t| t.verified_ticket().amount).sum::<HoprBalance>(),
            unrealized_value
        );

        let connector = std::sync::Arc::new(create_test_connector(&dst, &channel, None).await?);

        let results = mgr
            .redeem_stream(
                connector.clone(),
                *channel.get_id(),
                Some(tickets[0].verified_ticket().amount + 1),
            )?
            .try_collect::<Vec<_>>()
            .await?;

        assert_eq!(
            results,
            tickets
                .into_iter()
                .map(|t| RedemptionResult::ValueTooLow(t.ticket))
                .collect::<Vec<_>>()
        );

        Ok(())
    }
}
