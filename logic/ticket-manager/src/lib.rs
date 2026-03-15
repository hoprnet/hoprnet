mod backend;
mod errors;
mod traits;
mod utils;

use std::{ops::Mul, sync::atomic::AtomicU64};

use hopr_api::{
    chain::TicketRedeemError,
    types::{internal::prelude::*, primitive::prelude::*},
};
use utils::{ChannelTicketQueue, OutgoingIndexTracker};

#[cfg(feature = "redb")]
pub use crate::backend::{RedbStore, RedbTicketQueue};
pub use crate::{
    backend::{MemoryStore, MemoryTicketQueue, ValueCachedQueue},
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
/// persistence, others may not.
///
/// The node type gives typical use-cases of the `HoprTicketManager`:
///
/// - Entry/Exit nodes only need to provide an `OutgoingIndexStore`, since they are dealing with outgoing tickets only.
/// - Relay nodes need to provide a store which implements both `OutgoingIndexStore + TicketQueueStore`, because they
///   need to deal with both outgoing tickets and incoming redeemable tickets.
///
/// To synchronize the on-chain state with the store, it is advised to call
/// [`sync_outgoing_channels`](HoprTicketManager::sync_outgoing_channels) (and
/// [`sync_incoming_channels`](HoprTicketManager::sync_incoming_channels) if applicable to the chosen store) early after
/// the construction of the manager, to make sure outdated data is discarded early. This is typically done only once
/// after construction and not needed to be done during the life-time of the manager.
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
/// ticket index for a channel. The synchronization to the underlying storage is done periodically in the background,
/// making snapshots of the current.
/// No significant contention is expected.
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
/// one of the following (write) operations is performed in the same moment:
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
    out_idx_tracker: OutgoingIndexTracker,
    channel_tickets: dashmap::DashMap<ChannelId, ChannelTicketQueue<ValueCachedQueue<Q>>>,
    store: std::sync::Arc<parking_lot::RwLock<S>>,
}

impl<S, Q> HoprTicketManager<S, Q>
where
    S: OutgoingIndexStore + Send + Sync + 'static,
{
    /// Creates a new ticket manager instance given the desired `store`.
    ///
    /// It is advised to call [`HoprTicketManager::sync_outgoing_channels`] and
    /// [`HoprTicketManager::sync_incoming_channels`] at least once before the manager
    /// is used any further.
    pub fn new(store: S) -> Result<Self, TicketManagerError> {
        let store = std::sync::Arc::new(parking_lot::RwLock::new(store));
        let out_idx_tracker = OutgoingIndexTracker::new(store.clone());
        Ok(Self {
            out_idx_tracker,
            channel_tickets: dashmap::DashMap::new(),
            store,
        })
    }

    /// Gets the next usable ticket index for an outgoing ticket in the given channel and epoch.
    ///
    /// This operation is fast and does not immediately put the index into the [`OutgoingIndexStore`].
    fn next_outgoing_ticket_index(&self, channel_id: &ChannelId, epoch: u32) -> u64 {
        let res = self
            .out_idx_tracker
            .index_cache()
            .entry((*channel_id, epoch))
            .or_default()
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        // If this is the first index in this epoch,
        // remove the previous epoch from the map if any
        if res == 1 && epoch > 0 {
            self.out_idx_tracker.index_cache().remove(&(*channel_id, epoch - 1));
            tracing::trace!(%channel_id, epoch = epoch - 1, "removing previous epoch from outgoing index cache");
        }

        // Mark the index store as dirty so that it could be synced to the persistent
        // store on the next tick
        self.out_idx_tracker.set_dirty();
        res
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
    pub fn sync_outgoing_channels(&self, outgoing_channels: &[ChannelEntry]) -> Result<(), TicketManagerError> {
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
            self.out_idx_tracker
                .index_cache()
                .insert((*id, epoch), AtomicU64::new(out_index));
            self.out_idx_tracker.set_dirty();

            tracing::debug!(%id, epoch, out_index, "outgoing ticket index for channel");
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
    pub fn sync_incoming_channels(
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
    pub fn insert_incoming_ticket(&self, ticket: RedeemableTicket) -> Result<(), TicketManagerError> {
        let ticket_id = ticket.ticket_id();
        match self.channel_tickets.entry(ticket_id.id) {
            dashmap::Entry::Occupied(e) => {
                // High contention on this write lock is possible only when massive numbers of winning tickets
                // on the same channel are received, or if tickets on the same channel are being
                // rapidly redeemed or neglected.
                // Such a scenario is likely not realistic.
                e.get().queue.write().push(ticket).map_err(TicketManagerError::store)?;
                tracing::debug!(%ticket_id, "winning ticket on channel");
            }
            dashmap::Entry::Vacant(v) => {
                // A hypothetical chance of high contention on this write lock is
                // only possible when massive numbers of winning tickets on new unique channels are received.
                // Such a scenario is likely not realistic.
                let queue = self
                    .store
                    .write()
                    .open_or_create_queue(&ticket.ticket_id().id)
                    .map_err(TicketManagerError::store)?;

                // Wrap the queue with a ticket value cache adapter
                let mut queue = ValueCachedQueue::new(queue).map_err(TicketManagerError::store)?;

                // Should not happen
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

        Ok(())
    }

    /// Returns the total value of unredeemed tickets in the given channel.
    ///
    /// NOTE: The function is less efficient when the `min_index` is specified, as
    /// a full scan of the queue is required to calculate the unrealized value.
    pub fn unrealized_value(
        &self,
        channel_id: &ChannelId,
        epoch: u32,
        min_index: Option<u64>,
    ) -> Result<HoprBalance, TicketManagerError> {
        if let Some(ticket_queue) = self.channel_tickets.get(channel_id) {
            // There is low contention on this read lock, because write locks are acquired only
            // when a new winning ticket has been added, redeemed or neglected, all of which are fairly rare operations.
            Ok(ticket_queue
                .queue
                .read()
                .total_value(epoch, min_index)
                .map_err(TicketManagerError::store)?)
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
        epoch: u32,
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
            .filter(|ticket| {
                epoch == ticket.verified_ticket().channel_epoch && max_index <= ticket.verified_ticket().index
            })
            .is_some()
        {
            // Quickly perform pop and downgrade to lock not to block any readers
            let mut queue_write = parking_lot::RwLockUpgradableReadGuard::upgrade(queue_read);
            let maybe_ticket = queue_write.pop().map_err(TicketManagerError::store)?;
            queue_read = parking_lot::RwLockWriteGuard::downgrade_to_upgradable(queue_write);

            neglected_tickets.extend(maybe_ticket.map(|t| *t.verified_ticket()));
            tracing::debug!(%channel_id, %epoch, ?maybe_ticket, "neglected ticket in channel");
        }

        // Keep the queue in even if it is empty. The cleanup is done only on startup.
        tracing::debug!(%channel_id, %epoch, num_tickets = neglected_tickets.len(), "ticket neglection done in channel");
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
