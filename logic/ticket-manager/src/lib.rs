mod errors;

use hopr_api::types::{internal::prelude::*, primitive::prelude::*};
use parking_lot::RwLockUpgradableReadGuard;

use crate::errors::TicketManagerError;

pub trait TicketQueue {
    type Error: std::error::Error + Send + 'static;
    fn push(&mut self, ticket: RedeemableTicket) -> Result<(), Self::Error>;
    fn pop(&mut self) -> Result<RedeemableTicket, Self::Error>;
    fn peek(&self) -> Result<Option<RedeemableTicket>, Self::Error>;
    fn iter(&self) -> impl Iterator<Item = Result<RedeemableTicket, Self::Error>>;
}

struct ChannelTicketQueue<Q> {
    queue: std::sync::Arc<parking_lot::RwLock<Q>>,
    redeem_lock: std::sync::Arc<parking_lot::Mutex<()>>,
}

pub struct HoprTicketManager<C, Q> {
    channel_tickets: std::collections::HashMap<ChannelId, ChannelTicketQueue<Q>>,
    chain: C,
}

impl<C, Q> HoprTicketManager<C, Q>
where
    C: hopr_api::chain::ChainWriteTicketOperations + Clone + Send + Sync + 'static,
    Q: TicketQueue + Send + Sync + 'static,
{
    pub fn unrealized_value(
        &self,
        channel_id: &ChannelId,
    ) -> Result<HoprBalance, TicketManagerError<C::Error, Q::Error>> {
        if let Some(ticket_queue) = self.channel_tickets.get(channel_id) {
            Ok(ticket_queue
                .queue
                .read()
                .iter()
                .filter_map(|ticket| ticket.ok().map(|t| t.verified_ticket().amount))
                .sum())
        } else {
            Err(TicketManagerError::ChannelNotFound)
        }
    }

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
                                // TODO: pop for specific unrecoverable errors + update stats
                                // Redemption timeouts should also be passed up.
                                (Err(TicketManagerError::RedeemError(error)), false)
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
