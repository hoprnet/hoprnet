mod backend;
mod errors;
mod traits;

use std::sync::atomic::AtomicU64;

use futures::{FutureExt, StreamExt};
use hopr_api::{
    chain::TicketRedeemError,
    types::{internal::prelude::*, primitive::prelude::*},
};

pub use crate::{
    backend::{MemoryStore, MemoryTicketQueue, RedbStore, RedbTicketQueue},
    errors::TicketManagerError,
    traits::{OutgoingIndexStore, TicketQueue, TicketQueueStore},
};

struct ChannelTicketQueue<Q> {
    queue: std::sync::Arc<parking_lot::RwLock<Q>>,
    redeem_lock: std::sync::Arc<parking_lot::Mutex<()>>,
}

impl<Q> From<Q> for ChannelTicketQueue<Q> {
    fn from(queue: Q) -> Self {
        Self {
            queue: std::sync::Arc::new(parking_lot::RwLock::new(queue)),
            redeem_lock: std::sync::Arc::new(parking_lot::Mutex::new(())),
        }
    }
}

const OUT_INDEX_SYNC_INTERVAL: std::time::Duration = std::time::Duration::from_secs(30);

pub struct HoprTicketManager<S, Q> {
    channel_tickets: dashmap::DashMap<ChannelId, ChannelTicketQueue<Q>>,
    store: std::sync::Arc<parking_lot::RwLock<S>>,
    out_indices: std::sync::Arc<dashmap::DashMap<(ChannelId, u32), AtomicU64>>,
    sync_handle: hopr_async_runtime::AbortHandle,
}

impl<S, Q> HoprTicketManager<S, Q>
where
    S: OutgoingIndexStore + Send + Sync + 'static,
    Q: Send + Sync + 'static,
{
    /// Gets the next usable ticket index for an outgoing ticket in the given channel and epoch.
    pub fn next_outgoing_ticket_index(&self, channel_id: &ChannelId, epoch: u32) -> u64 {
        let res = self
            .out_indices
            .entry((*channel_id, epoch))
            .or_default()
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        // Remove the previous epoch from the map
        if res == 1 && epoch > 0 {
            self.out_indices.remove(&(*channel_id, epoch - 1));
        }
        res
    }
}

impl<S> HoprTicketManager<S, S::Queue>
where
    S: TicketQueueStore + OutgoingIndexStore + Send + Sync + 'static,
    S::Queue: Send + Sync + 'static,
{
    /// Creates a new manager and establishes the internal state based on the current
    /// state of incoming and outgoing own `channels` and the given data `store`.
    pub fn new(me: &Address, channels: &[ChannelEntry], mut store: S) -> Result<Self, TicketManagerError> {
        let channel_tickets = dashmap::DashMap::new();
        let out_indices = std::sync::Arc::new(dashmap::DashMap::new());
        let now = std::time::SystemTime::now();

        // Purge outdated outgoing indices
        let stored_indices = store.iter_outgoing_indices().collect::<Vec<_>>();
        for (channel_id, epoch) in stored_indices {
            if channels
                .iter()
                .filter(|c| c.status == ChannelStatus::Open)
                .find(|c| c.get_id() == &channel_id && c.channel_epoch == epoch)
                .is_none()
            {
                store
                    .delete_outgoing_index(&channel_id, epoch)
                    .map_err(TicketManagerError::store)?;
                tracing::debug!(%channel_id, epoch, "purging outdated outgoing index")
            }
        }

        // Purge outdated channel queues
        let stored_queues = store.iter_queues().collect::<Vec<_>>();
        for channel_id in stored_queues {
            if channels
                .iter()
                .filter(|c| !c.closure_time_passed(now))
                .find(|c| c.get_id() == &channel_id)
                .is_none()
            {
                store.delete_queue(&channel_id).map_err(TicketManagerError::store)?;
                tracing::debug!(%channel_id, "purging outdated incoming tickets queue")
            }
        }

        // Load outgoing indices and redeemable ticket queues for relevant channels
        for channel in channels {
            let id = channel.get_id();
            match channel.direction(me) {
                // We are only interested in tickets on incoming channels that are still open or not overdue
                Some(ChannelDirection::Incoming) => {
                    if !channel.closure_time_passed(now) {
                        // Either open an existing queue for that channel or create a new one
                        channel_tickets.insert(
                            *id,
                            ChannelTicketQueue::from(
                                store.open_or_create_queue(id).map_err(TicketManagerError::store)?,
                            ),
                        );
                    }
                }
                // We are only interested in tickets on outgoing channels that are still open
                Some(ChannelDirection::Outgoing) => {
                    if channel.status == ChannelStatus::Open {
                        // Either load a previously stored outgoing index or use the channel's ticket index as a
                        // fallback
                        let index = match store.load_outgoing_index(id, channel.channel_epoch) {
                            Ok(Some(out_index)) => out_index,
                            Ok(None) => 0,
                            Err(error) => {
                                tracing::error!(%error, %id, "failed to load outgoing index for channel, falling back to channel ticket index");
                                0
                            }
                        };

                        // Always use the maximum from the stored value and the current ticket index on the channel
                        out_indices.insert(
                            (*id, channel.channel_epoch),
                            AtomicU64::new(index.max(channel.ticket_index)),
                        );
                    }
                }
                _ => {} // Foreign, closed or an "overdue" channel
            }
        }

        // Start syncing of the outgoing ticket indices back to the storage
        // This is needed, because the outgoing ticket indices change rapidly, and therefore
        // the sync cannot be done per call to `next_outgoing_ticket_index`
        let store = std::sync::Arc::new(parking_lot::RwLock::new(store));
        let out_indices_sync = out_indices.clone();
        let store_sync = store.clone();
        let (stream, sync_handle) =
            futures::stream::abortable(futures_time::stream::interval(OUT_INDEX_SYNC_INTERVAL.into()));
        hopr_async_runtime::prelude::spawn(stream
            .for_each(move |_| {
                let out_indices_sync = out_indices_sync.clone();
                let store_sync = store_sync.clone();
                let all_items = out_indices_sync
                    .iter()
                    .map(|e| {
                        (
                            e.key().0,
                            e.key().1,
                            e.value().load(std::sync::atomic::Ordering::SeqCst),
                        )
                    })
                    .collect::<Vec<_>>();

                async move {
                    let res = hopr_async_runtime::prelude::spawn_blocking(move || {
                    let mut store_sync = store_sync.write();
                    for (channel_id, channel_epoch, index) in all_items {
                        if let Err(error) = store_sync.save_outgoing_index(&channel_id, channel_epoch, index) {
                            tracing::error!(%error, %channel_id, %channel_epoch, %index, "failed to save outgoing index");
                        }
                    }
                }).await;
                    if let Err(error) = res {
                        tracing::error!(%error, "failed to sync outgoing indices");
                    }
                }
        }).inspect(|_| tracing::debug!("syncing outgoing indices done")));

        Ok(Self {
            channel_tickets,
            out_indices,
            store,
            sync_handle,
        })
    }

    /// Inserts a new incoming winning redeemable ticket into the ticket manager.
    pub fn insert_incoming_ticket(&self, ticket: RedeemableTicket) -> Result<(), TicketManagerError> {
        match self.channel_tickets.entry(ticket.ticket_id().id) {
            dashmap::Entry::Occupied(e) => {
                e.get().queue.write().push(ticket).map_err(TicketManagerError::store)?;
            }
            dashmap::Entry::Vacant(v) => {
                let mut queue = self
                    .store
                    .write()
                    .open_or_create_queue(&ticket.ticket_id().id)
                    .map_err(TicketManagerError::store)?;
                // Should not happen
                if !queue.is_empty() {
                    return Err(TicketManagerError::Other(anyhow::anyhow!("queue not empty")));
                }

                queue.push(ticket).map_err(TicketManagerError::store)?;
                v.insert(ChannelTicketQueue {
                    queue: std::sync::Arc::new(parking_lot::RwLock::new(queue)),
                    redeem_lock: std::sync::Arc::new(parking_lot::Mutex::new(())),
                });
            }
        }

        // TODO: invalidate unrealized value
        Ok(())
    }

    /// Returns the total value of unredeemed tickets in the given channel.
    pub fn unrealized_value(
        &self,
        channel_id: &ChannelId,
        epoch: u32,
        min_index: Option<u64>,
    ) -> Result<HoprBalance, TicketManagerError> {
        // TODO: consider using a cache here
        let min_index = min_index.unwrap_or(0);
        if let Some(ticket_queue) = self.channel_tickets.get(channel_id) {
            Ok(ticket_queue
                .queue
                .read()
                .iter_unordered()
                .filter_map(|ticket| {
                    ticket
                        .ok()
                        .filter(|t| {
                            epoch == t.verified_ticket().channel_epoch && min_index <= t.verified_ticket().index
                        })
                        .map(|t| t.verified_ticket().amount)
                })
                .sum())
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

        while let Some(_) = queue_read.peek().map_err(TicketManagerError::store)?.filter(|ticket| {
            epoch == ticket.verified_ticket().channel_epoch && max_index <= ticket.verified_ticket().index
        }) {
            let mut queue_write = parking_lot::RwLockUpgradableReadGuard::upgrade(queue_read);
            if let Some(ticket) = queue_write.pop().map_err(TicketManagerError::store)? {
                neglected_tickets.push(*ticket.verified_ticket());
            }
            queue_read = parking_lot::RwLockWriteGuard::downgrade_to_upgradable(queue_write);
        }

        // Keep the queue in even if it is empty. The cleanup is done only on startup.

        Ok(neglected_tickets)
    }

    /// Creates a stream that redeems tickets in-order one by one in the given channel.
    ///
    /// If `min_redeem_value` is given, all the tickets that are lower than the given value are neglected in the
    /// process.
    ///
    /// If there's already an existing redeem stream for the channel, an error is returned without creating a new
    /// stream.
    ///
    /// Possible errors during redemption are passed up via the stream, so the caller may choose if
    /// they wish to continue redeeming tickets based on the encountered error.
    pub fn redeem_stream<C>(
        &self,
        chain: C,
        channel_id: &ChannelId,
        min_redeem_value: Option<HoprBalance>,
    ) -> Result<impl futures::Stream<Item = Result<VerifiedTicket, TicketManagerError>>, TicketManagerError>
    where
        C: hopr_api::chain::ChainWriteTicketOperations + Send + Sync + 'static,
    {
        struct RedeemState<Cc, Qq, St> {
            _lock: parking_lot::ArcMutexGuard<parking_lot::RawMutex, ()>,
            queue: std::sync::Arc<parking_lot::RwLock<Qq>>,
            chain: Cc,
            store: std::sync::Arc<St>,
            min_redeem_value: HoprBalance,
        }

        let initial_state = match self.channel_tickets.get(channel_id) {
            Some(ticket_queue) => RedeemState {
                _lock: ticket_queue
                    .redeem_lock
                    .try_lock_arc()
                    .ok_or(TicketManagerError::AlreadyRedeeming)?,
                queue: ticket_queue.queue.clone(),
                chain,
                store: self.store.clone(),
                min_redeem_value: min_redeem_value.unwrap_or(ChannelEntry::MAX_CHANNEL_BALANCE.into()),
            },
            None => return Err(TicketManagerError::ChannelNotFound),
        };

        Ok(futures::stream::try_unfold(initial_state, |state| async move {
            let res = {
                let queue_read = state.queue.upgradable_read();
                match queue_read.peek() {
                    Ok(Some(ticket)) => {
                        let (redeem_result, remove_ticket) =
                            if ticket.verified_ticket().amount >= state.min_redeem_value {
                                match state.chain.redeem_ticket(ticket).await {
                                    Ok(redeem_fut) => {
                                        // TODO: update stats + invalidate unrealized value cache?
                                        let redemption_result = redeem_fut.await;

                                        // See if we need to remove this ticket after the error
                                        let reject_ticket = match &redemption_result {
                                            Ok(_) => true,
                                            Err(TicketRedeemError::Rejected(..)) => {
                                                // TODO: update stats + invalidate unrealized value cache?
                                                true
                                            }
                                            Err(TicketRedeemError::ProcessingError(..)) => false,
                                        };

                                        (
                                            redemption_result
                                                .map(|(ticket, _)| Some(ticket))
                                                .map_err(TicketManagerError::redeem),
                                            reject_ticket,
                                        )
                                    }
                                    Err(error) => {
                                        // See if we need to remove this ticket after the error
                                        let reject_ticket = match &error {
                                            TicketRedeemError::Rejected(..) => {
                                                // TODO: update stats + invalidate unrealized value cache?
                                                true
                                            }
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
                            let _ = parking_lot::RwLockUpgradableReadGuard::upgrade(queue_read).pop();
                        }

                        redeem_result
                    }
                    Ok(None) => {
                        // No more tickets to redeem in this channel
                        // Keep the queue in even if it is empty. The cleanup is done only on startup.
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

impl<S, Q> Drop for HoprTicketManager<S, Q> {
    fn drop(&mut self) {
        self.sync_handle.abort();
    }
}
