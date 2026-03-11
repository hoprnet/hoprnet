mod errors;

use hopr_api::types::{internal::prelude::*, primitive::prelude::*};
use parking_lot::{ArcRwLockUpgradableReadGuard, ArcRwLockWriteGuard};
use crate::errors::TicketManagerError;


pub trait TicketQueue {
    type Error: std::error::Error + Send + 'static;
    fn push(&mut self, ticket: RedeemableTicket) -> Result<(), Self::Error>;
    fn pop(&mut self) -> Result<RedeemableTicket, Self::Error>;
    fn peek(&self) -> Result<Option<RedeemableTicket>, Self::Error>;
    fn iter(&self) -> impl Iterator<Item = Result<RedeemableTicket, Self::Error>>;
}

pub struct HoprTicketManager<C, Q> {
    channel_tickets: std::collections::HashMap<ChannelId, std::sync::Arc<parking_lot::RwLock<Q>>>,
    chain: C
}

impl<C, Q> HoprTicketManager<C, Q>
where
    C: hopr_api::chain::ChainWriteTicketOperations + Clone + Send + Sync + 'static,
    Q: TicketQueue + Send + Sync + 'static,
{
    pub fn unrealized_value(&self, channel_id: &ChannelId) -> Result<HoprBalance, TicketManagerError<C::Error, Q::Error>> {
        if let Some(ticket_queue) = self.channel_tickets.get(channel_id) {
            Ok(ticket_queue
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
    ) -> Result<impl futures::Stream<Item = Result<VerifiedTicket, TicketManagerError<C::Error, Q::Error>>>, TicketManagerError<C::Error, Q::Error>>
    {
        let Some(ticket_queue) = self.channel_tickets.get(channel_id).cloned() else {
            return Err(TicketManagerError::ChannelNotFound);
        };
        let Some(locked_queue) = ticket_queue.try_upgradable_read_arc() else {
            return Err(TicketManagerError::AlreadyRedeeming);
        };

        Ok(futures::stream::try_unfold(
            (locked_queue, self.chain.clone()),
            |(queue, chain): (ArcRwLockUpgradableReadGuard<_, Q>, C)| async move {
                match queue.peek() {
                    Ok(Some(ticket)) => {
                        let (redeem_result, remove_ticket) = match chain.redeem_ticket(ticket).await {
                            Ok(redeem_fut) => {
                                // TODO: update stats + invalidate unrealized value cache?
                                let redemption_result =
                                    redeem_fut
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
                        let mut next_state = if remove_ticket {
                            let mut queue = ArcRwLockUpgradableReadGuard::upgrade(queue);
                            let _ = queue.pop();
                            (ArcRwLockWriteGuard::downgrade_to_upgradable(queue), chain)
                        } else {
                            (queue, chain)
                        };
                        ArcRwLockUpgradableReadGuard::bump(&mut next_state.0);
                        redeem_result.map(|s| s.map(|v| (v, next_state)))
                    }
                    Ok(None) => {
                        // No more tickets to redeem in this channel
                        Ok(None)
                    }
                    Err(error) => {
                        // Pass errors from the queue
                        Err(TicketManagerError::QueueError(error))
                    },
                }
            },
        ))
    }
}
