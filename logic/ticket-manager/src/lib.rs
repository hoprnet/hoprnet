mod errors;
pub mod queue;

use hopr_api::{
    chain::TicketRedeemError,
    types::{internal::prelude::*, primitive::prelude::*},
};
use parking_lot::RwLockUpgradableReadGuard;

use crate::{errors::TicketManagerError, queue::TicketQueue};

struct ChannelTicketQueue<Q> {
    queue: std::sync::Arc<parking_lot::RwLock<Q>>,
    redeem_lock: std::sync::Arc<parking_lot::Mutex<()>>,
}

pub struct HoprTicketManager<C, Q> {
    channel_tickets: dashmap::DashMap<ChannelId, ChannelTicketQueue<Q>>,
    chain: C,
}

impl<C, Q> HoprTicketManager<C, Q>
where
    C: hopr_api::chain::ChainWriteTicketOperations + Clone + Send + Sync + 'static,
    Q: TicketQueue + Default + Send + Sync + 'static,
{
    /// Inserts a new winning redeemable ticket into the ticket manager.
    pub fn insert_ticket(&self, ticket: RedeemableTicket) -> Result<(), TicketManagerError<C::Error, Q::Error>> {
        match self.channel_tickets.entry(ticket.ticket_id().id) {
            dashmap::Entry::Occupied(e) => {
                e.get()
                    .queue
                    .write()
                    .push(ticket)
                    .map_err(TicketManagerError::QueueError)?;
            }
            dashmap::Entry::Vacant(v) => {
                let mut queue = Q::default();
                queue.push(ticket).map_err(TicketManagerError::QueueError)?;
                v.insert(ChannelTicketQueue {
                    queue: std::sync::Arc::new(parking_lot::RwLock::new(queue)),
                    redeem_lock: std::sync::Arc::new(parking_lot::Mutex::new(())),
                });
            }
        }
        Ok(())
    }

    /// Returns the total value of unredeemed tickets in the given channel.
    pub fn unrealized_value(
        &self,
        channel_id: &ChannelId,
    ) -> Result<HoprBalance, TicketManagerError<C::Error, Q::Error>> {
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

    /// Creates a stream that redeems tickets in-order one by one in the given channel.
    ///
    /// If there's already an existing redeem stream for the channel, an error is returned without creating a new stream.
    ///
    /// Possible errors during redemption are passed up via the stream.
    pub fn redeem_stream(
        &self,
        channel_id: &ChannelId,
    ) -> Result<
        impl futures::Stream<Item = Result<VerifiedTicket, TicketManagerError<C::Error, Q::Error>>>,
        TicketManagerError<C::Error, Q::Error>,
    > {
        struct RedeemState<Cc, Qq> {
            _lock: parking_lot::ArcMutexGuard<parking_lot::RawMutex, ()>,
            queue: std::sync::Arc<parking_lot::RwLock<Qq>>,
            chain: Cc,
        }

        let initial_state = match self.channel_tickets.get(channel_id) {
            Some(ticket_queue) => RedeemState {
                _lock: ticket_queue
                    .redeem_lock
                    .try_lock_arc()
                    .ok_or(TicketManagerError::AlreadyRedeeming)?,
                queue: ticket_queue.queue.clone(),
                chain: self.chain.clone(),
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
                                    .await
                                    .map(|(ticket, _)| Some(ticket))
                                    .map_err(TicketManagerError::RedeemError);
                                (redemption_result, true)
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
                                (Err(TicketManagerError::RedeemError(error)), reject_ticket)
                            }
                        };

                        // Check if we should remove the ticket from the queue
                        if remove_ticket {
                            let mut queue_write = RwLockUpgradableReadGuard::upgrade(queue_read);
                            let _ = queue_write.pop();
                        }

                        redeem_result
                    }
                    Ok(None) => {
                        // No more tickets to redeem in this channel
                        Ok(None)
                    }
                    Err(error) => {
                        // Pass errors from the queue
                        Err(TicketManagerError::QueueError(error))
                    }
                }
            };
            res.map(|s| s.map(|v| (v, state)))
        }))
    }
}
