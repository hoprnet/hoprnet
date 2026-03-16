//! Implements complete logic of ticket management in the HOPR protocol.
//!
//! See the [`HoprTicketManager`] documentation for complete details.

mod backend;
mod errors;
mod traits;
mod utils;

use std::ops::Mul;

use hopr_api::{
    chain::TicketRedeemError,
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
/// [`create_multihop_ticket`](HoprTicketManager::create_multihop_ticket) to create a ticket for the next hop on a
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
/// 1. redeeming them via [`redeem_stream`](HoprTicketManager::redeem_stream)
/// 2. neglecting them via [`neglect_tickets`](HoprTicketManager::neglect_tickets)
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
/// The [`create_multihop_ticket`](HoprTicketManager::create_multihop_ticket) method is designed to be
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
    fn next_outgoing_ticket_index(&self, channel_id: &ChannelId, epoch: u32) -> u64 {
        let next_index = self.out_idx_tracker.next(channel_id, epoch);
        tracing::trace!(%channel_id, epoch, next_index, "next outgoing ticket index");

        // If this is the first index in this epoch,
        // remove the previous epoch from the map if any
        if next_index == 1 && epoch > 0 && self.out_idx_tracker.remove(channel_id, epoch - 1) {
            tracing::trace!(%channel_id, prev_epoch = epoch - 1, "removing previous epoch from outgoing index cache");
        }

        next_index
    }

    /// Saves outgoing ticket indices back to the store.
    ///
    /// The operation does nothing if there were no [new tickets created](HoprTicketManager::create_multihop_ticket)
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
            self.out_idx_tracker.set(id, epoch, out_index);
            tracing::debug!(%id, epoch, out_index, "loaded outgoing ticket index for channel");
        }

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
    pub fn create_multihop_ticket(
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

        // The next ticket is worth: price * remaining hop count / winning probability
        let amount = HoprBalance::from(
            ticket_price
                .amount()
                .mul(U256::from(current_path_pos - 1))
                .div_f64(winning_prob.into())
                .expect("winning probability is always less than or equal to 1"),
        );

        if channel.balance.lt(&amount) {
            return Err(TicketManagerError::OutOfFunds(*channel.get_id(), amount));
        }

        let ticket_builder = TicketBuilder::default()
            .counterparty(channel.destination)
            .balance(amount)
            .index(self.next_outgoing_ticket_index(channel.get_id(), channel.channel_epoch))
            .win_prob(winning_prob)
            .channel_epoch(channel.channel_epoch);

        Ok(ticket_builder)
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
    ) -> Result<Vec<Ticket>, TicketManagerError> {
        // Purge outdated incoming channel queues
        let mut store_read = self.store.upgradable_read();
        let stored_queues = store_read
            .iter_queues()
            .map_err(TicketManagerError::store)?
            .collect::<Vec<_>>();
        let mut ret = Vec::new();
        let now = hopr_platform::time::current_time();
        for channel_id in stored_queues {
            // If any existing redeemable ticket queue does not match any currently existing
            // channel that's either open or its closure period did not yet elapse (i.e. the channel
            // is not closed or not effectively closed), remove the queue from the store.
            if !incoming_channels
                .iter()
                .any(|channel| !channel.closure_time_passed(now) && channel.get_id() == &channel_id)
            {
                let mut store_write = parking_lot::RwLockUpgradableReadGuard::upgrade(store_read);
                ret.extend(
                    store_write
                        .delete_queue(&channel_id)
                        .map_err(TicketManagerError::store)?,
                );
                tracing::debug!(%channel_id, "purged outdated incoming tickets queue");
                store_read = parking_lot::RwLockWriteGuard::downgrade_to_upgradable(store_write);
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

        Ok(ret)
    }

    /// Inserts a new incoming winning redeemable ticket into the ticket manager.
    ///
    /// On success, the method returns all tickets that have been neglected in the ticket queue of this channel,
    /// in case the inserted ticket has a greater channel epoch than the [next extractable](TicketQueue::peek) ticket in
    /// the queue. This situation can happen when unredeemed tickets are left in the queue, while the corresponding
    /// channel restarts its lifecycle and a new winning ticket is received.
    /// Otherwise, the returned vector is empty.
    pub fn insert_incoming_ticket(&self, ticket: RedeemableTicket) -> Result<Vec<Ticket>, TicketManagerError> {
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
                if queue
                    .peek()
                    .map_err(TicketManagerError::store)?
                    .is_some_and(|last| last.verified_ticket().channel_epoch < ticket.verified_ticket().channel_epoch)
                {
                    // Ensures allocation according to the number of drained tickets
                    neglected_tickets.append(&mut queue.drain().map_err(TicketManagerError::store)?);
                    tracing::warn!(%ticket_id, num_neglected = neglected_tickets.len(), "winning ticket has neglected unredeemed tickets from previous epochs");
                }
                queue.push(ticket).map_err(TicketManagerError::store)?;
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
                v.insert(ChannelTicketQueue {
                    queue: std::sync::Arc::new(parking_lot::RwLock::new(queue)),
                    redeem_lock: std::sync::Arc::new(parking_lot::Mutex::new(())),
                });
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
    ) -> Result<HoprBalance, TicketManagerError> {
        if let Some(ticket_queue) = self.channel_tickets.get(channel_id) {
            // There is low contention on this read lock, because write locks are acquired only
            // when a new winning ticket has been added, redeemed or neglected, all of which are fairly rare operations.
            let queue = ticket_queue.queue.read();

            // Get the epoch of the first extractable ticket in the queue.
            // The manager takes care that there are no tickets with epochs other than the current epoch.
            if let Some(epoch) = queue
                .peek()
                .map_err(TicketManagerError::store)?
                .map(|t| t.verified_ticket().channel_epoch)
            {
                Ok(queue.total_value(epoch, min_index).map_err(TicketManagerError::store)?)
            } else {
                Ok(HoprBalance::zero())
            }
        } else {
            Err(TicketManagerError::ChannelNotFound)
        }
    }

    /// Removes all the tickets in the given [`ChannelId`], optionally only up to the given ticket index (inclusive).
    ///
    /// If the `max_index` is given and lower than the lowest index of an unredeemed ticket in the queue,
    /// the function does nothing.
    ///
    /// If there's ticket redemption ongoing in the same channel, the operation will fail with
    /// [`TicketManagerError::AlreadyRedeeming`]. In that case the ongoing redemption will likely fail because of
    /// the changed state of the channel that likely triggered the neglection.
    pub fn neglect_tickets(
        &self,
        channel_id: &ChannelId,
        max_index: Option<u64>,
    ) -> Result<Vec<Ticket>, TicketManagerError> {
        let (_lock, queue) = match self.channel_tickets.get(channel_id) {
            None => return Err(TicketManagerError::ChannelNotFound),
            Some(queue) => {
                let lock = queue
                    .redeem_lock
                    .try_lock_arc()
                    .ok_or(TicketManagerError::AlreadyRedeeming)?;
                (lock, queue.queue.clone())
            }
        };

        let mut neglected_tickets = Vec::new();
        let mut queue_read = queue.upgradable_read();
        let max_index = max_index.unwrap_or(u64::MAX);

        while queue_read
            .peek()
            .map_err(TicketManagerError::store)?
            .filter(|ticket| max_index <= ticket.verified_ticket().index)
            .is_some()
        {
            // Quickly perform pop and downgrade to lock not to block any readers
            let mut queue_write = parking_lot::RwLockUpgradableReadGuard::upgrade(queue_read);
            let maybe_ticket = queue_write.pop().map_err(TicketManagerError::store)?;
            queue_read = parking_lot::RwLockWriteGuard::downgrade_to_upgradable(queue_write);

            neglected_tickets.extend(maybe_ticket.map(|t| *t.verified_ticket()));
            tracing::debug!(%channel_id, ?maybe_ticket, "neglected ticket in channel");
        }

        // Keep the queue in even if it is empty. The cleanup is done only on startup.
        tracing::debug!(%channel_id, num_tickets = neglected_tickets.len(), "ticket neglection done in channel");
        Ok(neglected_tickets)
    }

    /// Creates a stream that redeems tickets in-order one by one in the given channel,
    /// using the given [`ChainWriteTicketOperations`](hopr_api::chain::ChainWriteTicketOperations) on-chain client
    /// implementation.
    ///
    /// If `min_redeem_value` is given, all the tickets that are lower than the given value are neglected in the
    /// process.
    ///
    /// If there's already an existing redeem stream for the channel, an error is returned without creating a new
    /// stream.
    ///
    /// Possible errors during redemption are passed up via the stream, so the caller may choose if
    /// they wish to continue redeeming tickets based on the encountered error and/or to do any accounting.
    pub fn redeem_stream<C>(
        &self,
        chain: C,
        channel_id: &ChannelId,
        min_redeem_value: Option<HoprBalance>,
    ) -> Result<impl futures::Stream<Item = Result<VerifiedTicket, TicketManagerError>>, TicketManagerError>
    where
        C: hopr_api::chain::ChainWriteTicketOperations + Send + Sync + 'static,
    {
        struct RedeemState<Cc, Qq> {
            _lock: parking_lot::ArcMutexGuard<parking_lot::RawMutex, ()>,
            queue: std::sync::Arc<parking_lot::RwLock<Qq>>,
            chain: Cc,
            min_redeem_value: HoprBalance,
            channel_id: ChannelId,
        }

        let initial_state = match self.channel_tickets.get(channel_id) {
            Some(ticket_queue) => RedeemState {
                _lock: ticket_queue
                    .redeem_lock
                    .try_lock_arc()
                    .ok_or(TicketManagerError::AlreadyRedeeming)?,
                queue: ticket_queue.queue.clone(),
                chain,
                min_redeem_value: min_redeem_value.unwrap_or(ChannelEntry::MAX_CHANNEL_BALANCE.into()),
                channel_id: *channel_id,
            },
            None => return Err(TicketManagerError::ChannelNotFound),
        };

        Ok(futures::stream::try_unfold(initial_state, |state| async move {
            let res = {
                let queue_read = state.queue.upgradable_read();
                match queue_read.peek() {
                    Ok(Some(ticket)) => {
                        let ticket_id = ticket.ticket_id();
                        let (redeem_result, remove_ticket) =
                            if ticket.verified_ticket().amount >= state.min_redeem_value {
                                match state.chain.redeem_ticket(ticket).await {
                                    Ok(redeem_fut) => {
                                        let redemption_result = redeem_fut.await;

                                        // See if we need to remove this ticket after the error
                                        let pop_ticket = match &redemption_result {
                                            Ok(_) | Err(TicketRedeemError::Rejected(..)) => true,
                                            Err(TicketRedeemError::ProcessingError(..)) => false,
                                        };

                                        (
                                            redemption_result
                                                .map(|(ticket, _)| Some(ticket))
                                                .map_err(TicketManagerError::redeem),
                                            pop_ticket,
                                        )
                                    }
                                    Err(error) => {
                                        // See if we need to remove this ticket after the error
                                        let reject_ticket = match &error {
                                            TicketRedeemError::Rejected(..) => true,
                                            TicketRedeemError::ProcessingError(..) => false,
                                        };
                                        (Err(TicketManagerError::redeem(error)), reject_ticket)
                                    }
                                }
                            } else {
                                // Tickets with low value are treated as errors and discarded
                                (
                                    Err(TicketManagerError::TicketValueLow((*ticket.verified_ticket()).into())),
                                    true,
                                )
                            };

                        // Check if we should remove the ticket from the queue
                        if remove_ticket {
                            // Quickly perform pop and drop the write lock not to block any readers
                            let _ = parking_lot::RwLockUpgradableReadGuard::upgrade(queue_read).pop();
                        }

                        redeem_result
                            .inspect(|_| tracing::debug!(%ticket_id, "ticket redemption succeeded"))
                            .inspect_err(
                                |error| tracing::error!(%error, %ticket_id, remove_ticket, "ticket redemption failed"),
                            )
                    }
                    Ok(None) => {
                        // No more tickets to redeem in this channel
                        // Keep the queue in even if it is empty. The cleanup is done only on startup.
                        tracing::debug!(channel_id = %state.channel_id, "no more tickets to redeem in channel");
                        Ok(None)
                    }
                    Err(error) => {
                        // Pass errors from the queue
                        Err(TicketManagerError::store(error))
                    }
                }
            };
            res.map(|s| s.map(|v| (v, state)))
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use hopr_api::types::crypto::prelude::{ChainKeypair, Keypair};
    use crate::traits::tests::generate_tickets;

    #[test]
    fn ticket_manager_should_create_multihop_tickets() -> anyhow::Result<()> {
        let mgr: HoprTicketManager<_, MemoryTicketQueue> = HoprTicketManager::new(MemoryStore::default())?;

        let src = ChainKeypair::random();
        let dst = ChainKeypair::random();

        let channel = ChannelEntry::new(
            src.public().to_address(),
            dst.public().to_address(),
            10.into(),
            1_u32.into(),
            ChannelStatus::Open,
            1_u32.into()
        );

        // Loads index 1 which is the next index for a ticket on this channel
        mgr.sync_from_outgoing_channels(&[channel])?;

        let ticket = mgr.create_multihop_ticket(&channel, 2, WinningProbability::ALWAYS, 10.into())?
            .eth_challenge(Default::default())
            .build_signed(&src, &Default::default())?;

        assert_eq!(ticket.channel_id(), channel.get_id());
        assert_eq!(channel.ticket_index, ticket.verified_ticket().index);
        assert_eq!(channel.channel_epoch, ticket.verified_ticket().channel_epoch);

        Ok(())
    }

    #[test]
    fn ticket_manager_should_save_out_indices_to_the_store_on_demand() -> anyhow::Result<()> {
        let mgr: HoprTicketManager<_, MemoryTicketQueue> = HoprTicketManager::new(MemoryStore::default())?;

        let src = ChainKeypair::random();
        let dst = ChainKeypair::random();

        let channel = ChannelEntry::new(
            src.public().to_address(),
            dst.public().to_address(),
            10.into(),
            1_u32.into(),
            ChannelStatus::Open,
            1_u32.into()
        );

        // Loads index 1 which is the next index for a ticket on this channel
        mgr.sync_from_outgoing_channels(&[channel])?;

        mgr.create_multihop_ticket(&channel, 2, WinningProbability::ALWAYS, 10.into())?;

        // Without saving, the store index should not be present in store
        let saved_index = mgr.store.read().load_outgoing_index(channel.get_id(), 1)?;
        assert_eq!(None, saved_index);

        mgr.create_multihop_ticket(&channel, 2, WinningProbability::ALWAYS, 10.into())?;

        mgr.save_outgoing_indices()?;
        let saved_index = mgr.store.read().load_outgoing_index(channel.get_id(), 1)?;
        assert_eq!(Some(3), saved_index);

        mgr.create_multihop_ticket(&channel, 2, WinningProbability::ALWAYS, 10.into())?;

        let saved_index = mgr.store.read().load_outgoing_index(channel.get_id(), 1)?;
        assert_eq!(Some(3), saved_index);

        mgr.save_outgoing_indices()?;
        let saved_index = mgr.store.read().load_outgoing_index(channel.get_id(), 1)?;
        assert_eq!(Some(4), saved_index);

        Ok(())
    }

    #[test]
    fn ticket_manager_should_sync_out_indices_from_chain_state() -> anyhow::Result<()> {
        let mgr: HoprTicketManager<_, MemoryTicketQueue> = HoprTicketManager::new(MemoryStore::default())?;

        let src = ChainKeypair::random();
        let dst = ChainKeypair::random();

        let channel = ChannelEntry::new(
            src.public().to_address(),
            dst.public().to_address(),
            10.into(),
            1_u32.into(),
            ChannelStatus::Open,
            1_u32.into()
        );

        mgr.sync_from_outgoing_channels(&[channel])?;
        mgr.save_outgoing_indices()?;

        let saved_index = mgr.store.read().load_outgoing_index(channel.get_id(), 1)?;
        assert_eq!(Some(1), saved_index);

        Ok(())
    }

    #[test]
    fn ticket_manager_should_sync_incoming_channels_from_chain_state() -> anyhow::Result<()> {
        let mgr: HoprTicketManager<_, MemoryTicketQueue> = HoprTicketManager::new(MemoryStore::default())?;

        let src = ChainKeypair::random();
        let dst = ChainKeypair::random();

        let channel = ChannelEntry::new(
            src.public().to_address(),
            dst.public().to_address(),
            10.into(),
            1_u32.into(),
            ChannelStatus::Open,
            1_u32.into()
        );

        let neglected = mgr.sync_from_incoming_channels(&[channel])?;
        assert!(neglected.is_empty());

        let queues = mgr.store.read().iter_queues()?.collect::<Vec<_>>();
        assert_eq!(vec![*channel.get_id()], queues);

        Ok(())
    }

    #[test]
    fn ticket_manager_unrealized_value_should_increase_when_tickets_are_added() -> anyhow::Result<()> {
        let mut tickets = generate_tickets()?;
        let channel_id = tickets[0].ticket_id().id;
        let epoch = tickets[0].ticket_id().epoch;
        tickets.retain(|ticket| ticket.verified_ticket().channel_epoch == epoch);

        assert!(!tickets.is_empty());

        let mgr = HoprTicketManager::new(MemoryStore::default())?;
        assert!(matches!(
            mgr.unrealized_value(&channel_id, None),
            Err(TicketManagerError::ChannelNotFound)
        ));

        let mut last_unrealized_value = HoprBalance::zero();
        assert_eq!(HoprBalance::zero(), last_unrealized_value);

        for ticket in tickets.iter() {
            let neglected = mgr.insert_incoming_ticket(*ticket)?;
            assert!(neglected.is_empty());

            let new_unrealized_value = mgr.unrealized_value(&channel_id, None)?;
            assert_eq!(
                new_unrealized_value - last_unrealized_value,
                ticket.verified_ticket().amount
            );

            last_unrealized_value = new_unrealized_value;
        }

        let expected_unrealized_value: HoprBalance = tickets.iter().map(|ticket| ticket.verified_ticket().amount).sum();
        assert_eq!(expected_unrealized_value, last_unrealized_value);

        Ok(())
    }

    #[test]
    fn ticket_manager_ticket_insertion_should_neglect_tickets_from_previous_epochs() -> anyhow::Result<()> {
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

        let mgr = HoprTicketManager::new(MemoryStore::default())?;
        assert!(matches!(
            mgr.unrealized_value(&channel_id, None),
            Err(TicketManagerError::ChannelNotFound)
        ));

        for ticket in tickets_from_epoch_1.iter() {
            let neglected = mgr.insert_incoming_ticket(*ticket)?;
            assert!(neglected.is_empty());
        }

        let new_unrealized_value = mgr.unrealized_value(&channel_id, None)?;
        assert_eq!(
            new_unrealized_value,
            tickets_from_epoch_1
                .iter()
                .map(|ticket| ticket.verified_ticket().amount)
                .sum()
        );

        let neglected = mgr.insert_incoming_ticket(tickets_from_epoch_2[0].clone())?;
        assert_eq!(
            tickets_from_epoch_1
                .iter()
                .map(|t| *t.verified_ticket())
                .collect::<Vec<_>>(),
            neglected
        );

        // There's now only 1 ticket from epoch 2
        let new_unrealized_value = mgr.unrealized_value(&channel_id, None)?;
        assert_eq!(tickets_from_epoch_2[0].verified_ticket().amount, new_unrealized_value);

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
}
