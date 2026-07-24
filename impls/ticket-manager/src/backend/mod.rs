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
    // Epoch of tickets currently in the queue. All queued tickets share the same epoch
    // (invariant maintained by the ticket manager); this avoids calling peek() when
    // the current epoch is needed, which would otherwise require I/O on the redb backend.
    current_epoch: Option<u32>,
}

impl<Q: TicketQueue> ValueCachedQueue<Q> {
    pub fn new(queue: Q) -> Result<Self, Q::Error> {
        let mut value_cache = hashbrown::HashMap::<u32, HoprBalance>::new();
        let mut current_epoch: Option<u32> = None;
        // Load all pre-existing ticket values into the cache
        queue.iter_unordered()?.filter_map(|res| res.ok()).for_each(|ticket| {
            let epoch = ticket.verified_ticket().channel_epoch;
            current_epoch = Some(epoch);
            value_cache
                .entry(epoch)
                .or_default()
                .add_assign(ticket.verified_ticket().amount);
        });

        Ok(Self {
            queue,
            value_cache,
            current_epoch,
        })
    }

    /// Returns the total unrealized value of tickets in the queue without any I/O.
    ///
    /// Reads the cached value for the tracked current epoch directly from memory.
    /// Returns zero when the queue is empty or when no value has been accumulated
    /// for the current epoch (e.g. after all tickets have been popped).
    ///
    /// This is the fast path for [`crate::utils::UnrealizedValue::unrealized_value`]
    /// with `min_index = None` — it avoids calling [`TicketQueue::peek`] on the
    /// underlying queue, which may do I/O on persistent backends.
    pub fn cached_unrealized_value(&self) -> HoprBalance {
        self.current_epoch
            .and_then(|epoch| self.value_cache.get(&epoch))
            .copied()
            .unwrap_or_default()
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
        let epoch = ticket.verified_ticket().channel_epoch;
        self.current_epoch = Some(epoch.max(self.current_epoch.unwrap_or(0)));
        self.value_cache
            .entry(epoch)
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
            // Keep current_epoch consistent with what remains in the queue after the pop.
            // If the queue still has tickets, they share the same epoch; if empty, reset to None.
            self.current_epoch = self.queue.peek()?.map(|t| t.verified_ticket().channel_epoch);
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

    use crate::{
        backend::{ValueCachedQueue, memory},
        traits::tests::*,
    };

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

    /// Regression guard: `cached_unrealized_value` must never call the underlying
    /// queue's `peek()`. A cache miss on the redb backend opens a read transaction;
    /// calling it per packet would block the Rayon crypto worker on I/O.
    #[test]
    fn cached_unrealized_value_never_calls_peek() -> anyhow::Result<()> {
        use std::sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        };

        use hopr_api::chain::{HoprBalance, RedeemableTicket};

        use crate::TicketQueue;

        /// Spy that wraps `MemoryTicketQueue` and counts every `peek()` call.
        struct PeekSpy {
            inner: memory::MemoryTicketQueue,
            peek_count: Arc<AtomicUsize>,
        }

        impl TicketQueue for PeekSpy {
            type Error = <memory::MemoryTicketQueue as TicketQueue>::Error;

            fn len(&self) -> Result<usize, Self::Error> {
                self.inner.len()
            }

            fn push(&mut self, t: RedeemableTicket) -> Result<(), Self::Error> {
                self.inner.push(t)
            }

            fn pop(&mut self) -> Result<Option<RedeemableTicket>, Self::Error> {
                self.inner.pop()
            }

            fn peek(&self) -> Result<Option<RedeemableTicket>, Self::Error> {
                self.peek_count.fetch_add(1, Ordering::Relaxed);
                self.inner.peek()
            }

            fn iter_unordered(
                &self,
            ) -> Result<impl Iterator<Item = Result<RedeemableTicket, Self::Error>>, Self::Error> {
                self.inner.iter_unordered()
            }
        }

        let peek_count = Arc::new(AtomicUsize::new(0));
        let spy = PeekSpy {
            inner: memory::MemoryTicketQueue::default(),
            peek_count: peek_count.clone(),
        };
        let mut queue = ValueCachedQueue::new(spy)?;

        // Empty queue: cached_unrealized_value must return zero with no peek calls.
        assert_eq!(queue.cached_unrealized_value(), HoprBalance::default());
        assert_eq!(peek_count.load(Ordering::Relaxed), 0, "peek called on empty queue");

        // Push tickets from a single epoch to keep the test invariant clean.
        // generate_tickets() spans epochs 1..=2; we use only the highest epoch.
        let all_tickets = generate_tickets()?;
        let current_epoch = all_tickets
            .iter()
            .map(|t| t.verified_ticket().channel_epoch)
            .max()
            .unwrap_or(1);
        let tickets: Vec<_> = all_tickets
            .into_iter()
            .filter(|t| t.verified_ticket().channel_epoch == current_epoch)
            .collect();
        let expected_value: HoprBalance = tickets.iter().map(|t| t.verified_ticket().amount).sum();
        fill_queue(&mut queue, tickets.into_iter())?;

        for _ in 0..10 {
            assert_eq!(queue.cached_unrealized_value(), expected_value);
        }
        assert_eq!(
            peek_count.load(Ordering::Relaxed),
            0,
            "peek called during cached_unrealized_value"
        );

        Ok(())
    }
}
