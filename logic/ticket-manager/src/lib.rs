mod backend;
mod errors;
mod traits;

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

pub struct HoprTicketManager<S, Q> {
    channel_tickets: dashmap::DashMap<ChannelId, ChannelTicketQueue<Q>>,
    store: std::sync::Arc<parking_lot::RwLock<S>>,
}


impl<S, Q> HoprTicketManager<S, Q>
where
    S: OutgoingIndexStore + Send + Sync + 'static,
    Q: Send + Sync + 'static,
{
    pub fn next_outgoing_ticket_index(&self, channel_id: &ChannelId) -> Result<u64, TicketManagerError> {
        todo!()
    }

    pub fn reset_outgoing_ticket_index(&mut self, channel_id: &ChannelId) -> Result<(), TicketManagerError> {
        todo!()
    }
}

impl<S> HoprTicketManager<S, S::Queue>
where
    S: TicketQueueStore + Send + Sync + 'static,
    S::Queue: Send + Sync + 'static,
{
    /// Creates a new ticket manager backed by the given ticket queue store and attempts
    /// to open existing ticket queues.
    pub fn new(mut store: S) -> Result<Self, TicketManagerError> {
        let existing_channel_queues = store.iter_channels().collect::<Vec<_>>();
        Ok(Self {
            channel_tickets: existing_channel_queues
                .into_iter()
                .map(|c| store.open_or_create(&c).map(|q| (c, q.into())))
                .collect::<Result<dashmap::DashMap<_, _>, _>>()
                .map_err(TicketManagerError::queue)?,
            store: std::sync::Arc::new(parking_lot::RwLock::new(store)),
        })
    }

    /// Inserts a new incoming winning redeemable ticket into the ticket manager.
    pub fn insert_incoming_ticket(&self, ticket: RedeemableTicket) -> Result<(), TicketManagerError> {
        match self.channel_tickets.entry(ticket.ticket_id().id) {
            dashmap::Entry::Occupied(e) => {
                e.get().queue.write().push(ticket).map_err(TicketManagerError::queue)?;
            }
            dashmap::Entry::Vacant(v) => {
                let mut queue = self
                    .store
                    .write()
                    .open_or_create(&ticket.ticket_id().id)
                    .map_err(TicketManagerError::queue)?;
                // Should not happen
                if !queue.is_empty() {
                    return Err(TicketManagerError::Other(anyhow::anyhow!("queue not empty")));
                }

                queue.push(ticket).map_err(TicketManagerError::queue)?;
                v.insert(ChannelTicketQueue {
                    queue: std::sync::Arc::new(parking_lot::RwLock::new(queue)),
                    redeem_lock: std::sync::Arc::new(parking_lot::Mutex::new(())),
                });
            }
        }
        Ok(())
    }

    /// Returns the total value of unredeemed tickets in the given channel.
    pub fn unrealized_value(&self, channel_id: &ChannelId) -> Result<HoprBalance, TicketManagerError> {
        // TODO: consider using a cache here
        if let Some(ticket_queue) = self.channel_tickets.get(channel_id) {
            Ok(ticket_queue
                .queue
                .read()
                .iter_unordered()
                .filter_map(|ticket| ticket.ok().map(|t| t.verified_ticket().amount))
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
    /// If there's ticket redemption ongoing in the same channel, the operation will block
    /// until the redemption is done.
    pub fn neglect_tickets(&self, channel_id: &ChannelId, max_index: Option<u64>) -> Result<Vec<Ticket>, TicketManagerError> {
        let (_lock, queue) = match self.channel_tickets.get(channel_id) {
            None => return Err(TicketManagerError::ChannelNotFound),
            Some(queue) => {
                let lock = queue.redeem_lock.lock_arc();
                (lock, queue.queue.clone())
            }
        };

        let mut neglected_tickets = Vec::new();
        let mut queue_read = queue.upgradable_read();
        let max_index = max_index.unwrap_or(u64::MAX);
        
        while let Some(ticket) = queue_read.peek().map_err(TicketManagerError::queue)? {
            if max_index <= ticket.verified_ticket().index {
                let mut queue_write = parking_lot::RwLockUpgradableReadGuard::upgrade(queue_read);
                if let Some(ticket) = queue_write.pop().map_err(TicketManagerError::queue)? {
                    neglected_tickets.push(*ticket.verified_ticket());
                }
                queue_read = parking_lot::RwLockWriteGuard::downgrade_to_upgradable(queue_write);
            } else {
                break;
            }
        }


        Ok(neglected_tickets)
    }

    /// Creates a stream that redeems tickets in-order one by one in the given channel.
    ///
    /// If there's already an existing redeem stream for the channel, an error is returned without creating a new
    /// stream.
    ///
    /// Possible errors during redemption are passed up via the stream.
    pub fn redeem_stream<C>(
        &self,
        chain: C,
        channel_id: &ChannelId,
    ) -> Result<impl futures::Stream<Item = Result<VerifiedTicket, TicketManagerError>>, TicketManagerError>
    where
        C: hopr_api::chain::ChainWriteTicketOperations + Send + Sync + 'static,
    {
        struct RedeemState<Cc, Qq, St> {
            _lock: parking_lot::ArcMutexGuard<parking_lot::RawMutex, ()>,
            queue: std::sync::Arc<parking_lot::RwLock<Qq>>,
            chain: Cc,
            store: std::sync::Arc<St>,
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
            },
            None => return Err(TicketManagerError::ChannelNotFound),
        };

        Ok(futures::stream::try_unfold(initial_state, |state| async move {
            let res = {
                let queue_read = state.queue.upgradable_read();
                match queue_read.peek() {
                    Ok(Some(ticket)) => {
                        let (redeem_result, remove_ticket) = match state.chain.redeem_ticket(ticket).await {
                            Ok(redeem_fut) => {
                                // TODO: update stats + invalidate unrealized value cache?
                                let redemption_result = redeem_fut
                                    .await;
                                
                                // See if we need to remove this ticket after the error
                                let reject_ticket = match &redemption_result {
                                    Ok(_) => true,
                                    Err(TicketRedeemError::Rejected(..)) => {
                                        // TODO: update stats + invalidate unrealized value cache?
                                        true
                                    }
                                    Err(TicketRedeemError::ProcessingError(..)) => false,
                                };
                                
                                (redemption_result.map(|(ticket, _)| Some(ticket)).map_err(TicketManagerError::redeem), reject_ticket)
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
                        };

                        // Check if we should remove the ticket from the queue
                        if remove_ticket {
                            let _ = parking_lot::RwLockUpgradableReadGuard::upgrade(queue_read).pop();
                        }

                        redeem_result
                    }
                    Ok(None) => {
                        // No more tickets to redeem in this channel
                        Ok(None)
                    }
                    Err(error) => {
                        // Pass errors from the queue
                        Err(TicketManagerError::queue(error))
                    }
                }
            };
            res.map(|s| s.map(|v| (v, state)))
        }))
    }
}
