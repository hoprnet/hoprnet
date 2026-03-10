mod errors;

use hopr_api::types::{internal::prelude::*, primitive::prelude::*};
use parking_lot::{ArcRwLockUpgradableReadGuard, ArcRwLockWriteGuard};

type ChannelTickets = mmap_fifo::MmapFifo<RedeemableTicket, mmap_fifo::PostcardSerializer<RedeemableTicket>>;

pub struct HoprTicketManager {
    channel_tickets: std::collections::HashMap<ChannelId, std::sync::Arc<parking_lot::RwLock<ChannelTickets>>>,
}

impl HoprTicketManager {
    pub fn unrealized_value(&self, channel_id: &ChannelId) -> Result<HoprBalance, errors::TicketManagerError> {
        if let Some(ticket_queue) = self.channel_tickets.get(channel_id) {
            Ok(ticket_queue
                .read()
                .iter()
                .filter_map(|ticket| ticket.ok().map(|t| t.verified_ticket().amount))
                .sum())
        } else {
            Err(errors::TicketManagerError::ChannelNotFound)
        }
    }

    pub fn redeem_stream<F, Fut>(
        &self,
        channel_id: &ChannelId,
        redeem_fut: F,
    ) -> Result<impl futures::Stream<Item = Result<Ticket, errors::TicketManagerError>>, errors::TicketManagerError>
    where
        F: FnMut(RedeemableTicket) -> Fut + Send,
        Fut: futures::Future<Output = Result<Ticket, errors::TicketManagerError>> + Send,
    {
        let Some(ticket_queue) = self.channel_tickets.get(channel_id).cloned() else {
            return Err(errors::TicketManagerError::ChannelNotFound);
        };
        let Some(locked_queue) = ticket_queue.try_upgradable_read_arc() else {
            return Err(errors::TicketManagerError::AlreadyRedeeming);
        };

        Ok(futures::stream::unfold(
            (locked_queue, redeem_fut),
            |(queue, mut redeem_fut): (ArcRwLockUpgradableReadGuard<_, ChannelTickets>, F)| async move {
                match queue.peek() {
                    Ok(Some(ticket)) => {
                        let (res, remove_ticket) = match redeem_fut(ticket).await {
                            Ok(redeemed_ticket) => {
                                // TODO: update stats + invalidate unrealized value cache?
                                (Some(Ok(redeemed_ticket)), true)
                            }
                            Err(error) => {
                                // TODO: pop for specific unrecoverable errors + update stats
                                (Some(Err(error)), false)
                            }
                        };
                        if remove_ticket {
                            let mut queue = ArcRwLockUpgradableReadGuard::upgrade(queue);
                            let _ = queue.pop();
                            res.map(|v| (v, (ArcRwLockWriteGuard::downgrade_to_upgradable(queue), redeem_fut)))
                        } else {
                            res.map(|v| (v, (queue, redeem_fut)))
                        }
                    }
                    Ok(None) => {
                        // No more tickets to redeem in this channel
                        None
                    }
                    Err(error) => {
                        // Pass error from the queue
                        Some((Err(error.into()), (queue, redeem_fut)))
                    },
                }
            },
        ))
    }
}
