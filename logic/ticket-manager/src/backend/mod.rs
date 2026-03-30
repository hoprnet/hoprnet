mod memory;

#[cfg(feature = "redb")]
mod redb;

use std::ops::{AddAssign, SubAssign};

use hopr_api::chain::{HoprBalance, RedeemableTicket};
pub use memory::*;
#[cfg(feature = "redb")]
pub use redb::*;

use crate::{TicketQueue, traits::default_total_value};

/// Adapter for [`TicketQueue`] that caches the total ticket value per channel epoch.
///
/// The cache value is updated with each `push` and `pop` operation.
/// The [`total_value`](TicketQueue::total_value) method
/// returns the cached value if available, otherwise it delegates to the underlying queue.
/// If `min_index` is provided, the cache is bypassed and the underlying queue is queried directly.
///
/// All other calls are simply delegated to the underlying queue.
///
/// The implementation uses `hashbrown::HashMap` for efficient key-value storage.
#[derive(Clone, Debug)]
pub struct ValueCachedQueue<Q> {
    queue: Q,
    // Caches total ticket value per channel epoch
    value_cache: hashbrown::HashMap<u32, HoprBalance>,
}

impl<Q: TicketQueue> ValueCachedQueue<Q> {
    pub fn new(queue: Q) -> Result<Self, Q::Error> {
        let mut value_cache = hashbrown::HashMap::<u32, HoprBalance>::new();
        // Load all pre-existing ticket values into the cache
        queue.iter_unordered()?.filter_map(|res| res.ok()).for_each(|ticket| {
            value_cache
                .entry(ticket.verified_ticket().channel_epoch)
                .or_default()
                .add_assign(ticket.verified_ticket().amount);
        });

        Ok(Self { queue, value_cache })
    }
}

impl<Q: TicketQueue> TicketQueue for ValueCachedQueue<Q> {
    type Error = Q::Error;

    fn len(&self) -> Result<usize, Self::Error> {
        self.queue.len()
    }

    fn is_empty(&self) -> Result<bool, Self::Error> {
        self.queue.is_empty()
    }

    fn push(&mut self, ticket: RedeemableTicket) -> Result<(), Self::Error> {
        self.value_cache
            .entry(ticket.verified_ticket().channel_epoch)
            .or_default()
            .add_assign(ticket.verified_ticket().amount);
        self.queue.push(ticket)
    }

    fn pop(&mut self) -> Result<Option<RedeemableTicket>, Self::Error> {
        let ticket = self.queue.pop()?;
        if let Some(ticket) = &ticket {
            // NOTE: that all the arithmetic operations on the `HoprBalance` type are naturally saturating.
            self.value_cache
                .entry(ticket.verified_ticket().channel_epoch)
                .or_default()
                .sub_assign(ticket.verified_ticket().amount);
        }
        Ok(ticket)
    }

    fn peek(&self) -> Result<Option<RedeemableTicket>, Self::Error> {
        self.queue.peek()
    }

    fn iter_unordered(&self) -> Result<impl Iterator<Item = Result<RedeemableTicket, Self::Error>>, Self::Error> {
        self.queue.iter_unordered()
    }

    fn total_value(&self, epoch: u32, min_index: Option<u64>) -> Result<HoprBalance, Self::Error> {
        if min_index.is_none()
            && let Some(value) = self.value_cache.get(&epoch)
        {
            return Ok(*value);
        }

        default_total_value(&self.queue, epoch, min_index)
    }
}

#[cfg(test)]
pub mod tests {
    use std::ops::AddAssign;

    use hopr_api::chain::HoprBalance;

    use crate::{ValueCachedQueue, backend::memory, traits::tests::*};

    #[test]
    fn value_cached_queue_maintains_natural_ticket_order() -> anyhow::Result<()> {
        queue_maintains_natural_ticket_order(ValueCachedQueue::new(memory::MemoryTicketQueue::default())?)
    }

    #[test]
    fn value_cached_queue_returns_all_tickets() -> anyhow::Result<()> {
        queue_returns_all_tickets(ValueCachedQueue::new(memory::MemoryTicketQueue::default())?)
    }

    #[test]
    fn value_cached_queue_is_empty_when_drained() -> anyhow::Result<()> {
        queue_is_empty_when_drained(ValueCachedQueue::new(memory::MemoryTicketQueue::default())?)
    }

    #[test]
    fn value_cached_queue_returns_empty_iterator_when_drained() -> anyhow::Result<()> {
        queue_returns_empty_iterator_when_drained(ValueCachedQueue::new(memory::MemoryTicketQueue::default())?)
    }

    #[test]
    fn value_cached_queue_returns_correct_total_ticket_value() -> anyhow::Result<()> {
        queue_returns_correct_total_ticket_value(ValueCachedQueue::new(memory::MemoryTicketQueue::default())?)
    }

    #[test]
    fn value_cached_queue_returns_correct_total_ticket_value_with_min_index() -> anyhow::Result<()> {
        queue_returns_correct_total_ticket_value_with_min_index(ValueCachedQueue::new(
            memory::MemoryTicketQueue::default(),
        )?)
    }

    #[test]
    fn value_cache_queue_populates_cache_with_existing_tickets() -> anyhow::Result<()> {
        let tickets = generate_tickets()?;
        let mut queue_1 = memory::MemoryTicketQueue::default();
        fill_queue(&mut queue_1, tickets.iter().copied())?;

        let mut total_value_per_epoch = hashbrown::HashMap::<u32, HoprBalance>::new();
        tickets.into_iter().for_each(|ticket| {
            total_value_per_epoch
                .entry(ticket.verified_ticket().channel_epoch)
                .or_default()
                .add_assign(ticket.verified_ticket().amount);
        });

        let queue_2 = ValueCachedQueue::new(queue_1)?;
        assert_eq!(total_value_per_epoch, queue_2.value_cache);

        Ok(())
    }
}
