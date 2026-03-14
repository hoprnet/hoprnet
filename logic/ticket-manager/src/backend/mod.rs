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

impl<Q: TicketQueue> From<Q> for ValueCachedQueue<Q> {
    fn from(queue: Q) -> Self {
        let mut value_cache = hashbrown::HashMap::<u32, HoprBalance>::new();
        // Load all pre-existing ticket values into the cache
        queue.iter_unordered().filter_map(|res| res.ok()).for_each(|ticket| {
            value_cache
                .entry(ticket.verified_ticket().channel_epoch)
                .or_default()
                .add_assign(ticket.verified_ticket().amount);
        });

        Self { queue, value_cache }
    }
}

impl<Q: TicketQueue> TicketQueue for ValueCachedQueue<Q> {
    type Error = Q::Error;

    fn len(&self) -> usize {
        self.queue.len()
    }

    fn is_empty(&self) -> bool {
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

    fn iter_unordered(&self) -> impl Iterator<Item = Result<RedeemableTicket, Self::Error>> {
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

    use hopr_api::{
        chain::{HoprBalance, RedeemableTicket, WinningProbability},
        types::{crypto::prelude::*, crypto_random::Randomizable, internal::prelude::TicketBuilder},
    };
    use hopr_api::chain::ChannelId;
    use hopr_api::types::chain::exports::alloy::serde::JsonStorageKey::Hash;
    use rand::prelude::SliceRandom;

    use crate::{TicketQueue, ValueCachedQueue, backend::memory, OutgoingIndexStore, TicketQueueStore};

    const TICKET_VALUE: u64 = 10;

    fn generate_tickets() -> anyhow::Result<Vec<RedeemableTicket>> {
        let src = ChainKeypair::random();
        let dst = ChainKeypair::random();
        let mut tickets = Vec::new();

        for epoch in 1..=2 {
            for i in 0..5_u64 {
                let hk1 = HalfKey::random();
                let hk2 = HalfKey::random();

                let ticket = TicketBuilder::default()
                    .counterparty(&dst)
                    .index(i)
                    .channel_epoch(epoch)
                    .win_prob(WinningProbability::ALWAYS)
                    .amount(TICKET_VALUE)
                    .challenge(Challenge::from_hint_and_share(
                        &hk1.to_challenge()?,
                        &hk2.to_challenge()?,
                    )?)
                    .build_signed(&src, &Default::default())?
                    .into_acknowledged(Response::from_half_keys(&hk1, &hk2)?)
                    .into_redeemable(&dst, &Default::default())?;

                tickets.push(ticket);
            }
        }

        Ok(tickets)
    }

    fn fill_queue<Q: TicketQueue, I: Iterator<Item = RedeemableTicket>>(queue: &mut Q, iter: I) -> anyhow::Result<()> {
        for ticket in iter {
            queue.push(ticket)?;
        }
        Ok(())
    }

    pub fn queue_maintains_natural_ticket_order<Q: TicketQueue>(mut queue: Q) -> anyhow::Result<()> {
        let mut tickets = generate_tickets()?;
        tickets.shuffle(&mut rand::rng());

        fill_queue(&mut queue, tickets.iter().copied())?;

        let mut collected_tickets = Vec::new();
        while let Some(ticket) = queue.pop()? {
            collected_tickets.push(ticket);
        }

        tickets.sort();
        assert_eq!(collected_tickets, tickets);

        Ok(())
    }

    #[test]
    fn value_cached_queue_maintains_natural_ticket_order() -> anyhow::Result<()> {
        queue_maintains_natural_ticket_order(ValueCachedQueue::from(memory::MemoryTicketQueue::default()))
    }

    pub fn queue_returns_all_tickets<Q: TicketQueue>(mut queue: Q) -> anyhow::Result<()> {
        let mut tickets = generate_tickets()?;
        tickets.sort();

        fill_queue(&mut queue, tickets.iter().copied())?;

        let mut collected_tickets = queue.iter_unordered().filter_map(|r| r.ok()).collect::<Vec<_>>();
        collected_tickets.sort();

        assert_eq!(tickets, collected_tickets);
        Ok(())
    }

    #[test]
    fn value_cached_queue_returns_all_tickets() -> anyhow::Result<()> {
        queue_returns_all_tickets(ValueCachedQueue::from(memory::MemoryTicketQueue::default()))
    }

    pub fn queue_is_empty_when_drained<Q: TicketQueue>(mut queue: Q) -> anyhow::Result<()> {
        fill_queue(&mut queue, generate_tickets()?.into_iter())?;
        assert!(!queue.is_empty());

        while let Some(_) = queue.pop()? {}
        assert!(queue.is_empty());
        Ok(())
    }

    #[test]
    fn value_cached_queue_is_empty_when_drained() -> anyhow::Result<()> {
        queue_is_empty_when_drained(ValueCachedQueue::from(memory::MemoryTicketQueue::default()))
    }

    pub fn queue_returns_empty_iterator_when_drained<Q: TicketQueue>(mut queue: Q) -> anyhow::Result<()> {
        fill_queue(&mut queue, generate_tickets()?.into_iter())?;
        assert!(!queue.is_empty());

        while let Some(_) = queue.pop()? {}
        assert_eq!(queue.iter_unordered().count(), 0);
        Ok(())
    }

    #[test]
    fn value_cached_queue_returns_empty_iterator_when_drained() -> anyhow::Result<()> {
        queue_returns_empty_iterator_when_drained(ValueCachedQueue::from(memory::MemoryTicketQueue::default()))
    }

    pub fn queue_returns_correct_total_ticket_value<Q: TicketQueue>(mut queue: Q) -> anyhow::Result<()> {
        let tickets = generate_tickets()?;
        fill_queue(&mut queue, tickets.iter().copied())?;

        let expected_total_value: HoprBalance = tickets
            .into_iter()
            .filter(|ticket| ticket.verified_ticket().channel_epoch == 2)
            .map(|ticket| ticket.verified_ticket().amount)
            .sum();
        let actual_total_value = queue.total_value(2, None)?;

        assert_eq!(expected_total_value, actual_total_value);
        Ok(())
    }

    #[test]
    fn value_cached_queue_returns_correct_total_ticket_value() -> anyhow::Result<()> {
        queue_returns_correct_total_ticket_value(ValueCachedQueue::from(memory::MemoryTicketQueue::default()))
    }

    pub fn queue_returns_correct_total_ticket_value_with_min_index<Q: TicketQueue>(mut queue: Q) -> anyhow::Result<()> {
        let tickets = generate_tickets()?;
        fill_queue(&mut queue, tickets.iter().copied())?;

        let expected_total_value: HoprBalance = tickets
            .into_iter()
            .filter(|ticket| ticket.verified_ticket().channel_epoch == 2 && ticket.verified_ticket().index >= 2)
            .map(|ticket| ticket.verified_ticket().amount)
            .sum();
        let actual_total_value = queue.total_value(2, Some(2))?;

        assert_eq!(expected_total_value, actual_total_value);
        Ok(())
    }

    #[test]
    fn value_cached_queue_returns_correct_total_ticket_value_with_min_index() -> anyhow::Result<()> {
        queue_returns_correct_total_ticket_value_with_min_index(ValueCachedQueue::from(
            memory::MemoryTicketQueue::default(),
        ))
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

        let queue_2 = ValueCachedQueue::from(queue_1);
        assert_eq!(total_value_per_epoch, queue_2.value_cache);

        Ok(())
    }

    pub fn ticket_store_should_create_new_queue_for_channel<S: TicketQueueStore>(mut store: S) -> anyhow::Result<()> {
        assert_eq!(0, store.iter_queues().count());

        let _ = store.open_or_create_queue(&Default::default())?;
        let queues = store.iter_queues().collect::<Vec<_>>();
        assert_eq!(1, queues.len());
        assert!(queues.contains(&Default::default()));

        Ok(())
    }

    pub fn ticket_store_should_open_existing_queue_for_channel<S: TicketQueueStore>(mut store: S) -> anyhow::Result<()> {
        assert_eq!(0, store.iter_queues().count());

        let mut tickets = generate_tickets()?;
        let mut queue = store.open_or_create_queue(&Default::default())?;
        fill_queue(&mut queue, tickets.iter().copied())?;
        tickets.sort();

        let queue = store.open_or_create_queue(&Default::default())?;
        let opened_tickets = queue.iter_unordered().filter_map(|r| r.ok()).collect::<Vec<_>>();

        assert_eq!(tickets, opened_tickets);

        Ok(())
    }

    pub fn ticket_store_should_delete_existing_queue_for_channel<S: TicketQueueStore>(mut store: S) -> anyhow::Result<()> {
        assert_eq!(0, store.iter_queues().count());

        let _ = store.open_or_create_queue(&Default::default())?;
        let queues = store.iter_queues().collect::<Vec<_>>();
        assert_eq!(1, queues.len());
        assert!(queues.contains(&Default::default()));

        let _ = store.delete_queue(&Default::default())?;
        let queues = store.iter_queues().collect::<Vec<_>>();
        assert_eq!(0, queues.len());

        Ok(())
    }

    pub fn ticket_store_should_iterate_existing_queues_for_channel<S: TicketQueueStore>(mut store: S) -> anyhow::Result<()> {
        assert_eq!(0, store.iter_queues().count());

        let c1 = ChannelId::create(&[b"beef"]);
        let _ = store.open_or_create_queue(&c1)?;
        let queues = store.iter_queues().collect::<Vec<_>>();
        assert_eq!(1, queues.len());
        assert!(queues.contains(&c1));

        let c2 = ChannelId::create(&[b"feed"]);
        let _ = store.open_or_create_queue(&c2)?;
        let queues = store.iter_queues().collect::<Vec<_>>();

        assert_eq!(2, queues.len());
        assert!(queues.contains(&c1));
        assert!(queues.contains(&c2));

        Ok(())
    }

    pub fn ticket_store_should_not_fail_to_delete_non_existing_queue_for_channel<S: TicketQueueStore>(mut store: S) -> anyhow::Result<()> {
        assert_eq!(0, store.iter_queues().count());
        assert!(store.delete_queue(&Default::default()).is_ok());

        Ok(())
    }

    pub fn out_index_store_should_load_existing_index_for_channel_epoch<S: OutgoingIndexStore>(mut store: S) -> anyhow::Result<()> {
        store.save_outgoing_index(&Default::default(), 1, 1)?;
        let loaded = store.load_outgoing_index(&Default::default(), 1)?;
        assert_eq!(Some(1), loaded);
        Ok(())
    }

    pub fn out_index_store_should_not_load_non_existing_index_for_channel_epoch<S: OutgoingIndexStore>(mut store: S) -> anyhow::Result<()> {
        let loaded = store.load_outgoing_index(&Default::default(), 1)?;
        assert_eq!(None, loaded);
        Ok(())
    }

    pub fn out_index_store_should_delete_existing_index_for_channel_epoch<S: OutgoingIndexStore>(mut store: S) -> anyhow::Result<()> {
        store.save_outgoing_index(&Default::default(), 1, 1)?;
        let loaded = store.load_outgoing_index(&Default::default(), 1)?;
        assert_eq!(Some(1), loaded);
        store.delete_outgoing_index(&Default::default(), 1)?;
        let loaded = store.load_outgoing_index(&Default::default(), 1)?;
        assert_eq!(None, loaded);
        Ok(())
    }

    pub fn out_index_store_should_store_new_index_for_channel_epoch<S: OutgoingIndexStore>(mut store: S) -> anyhow::Result<()> {
        store.save_outgoing_index(&Default::default(), 1, 1)?;
        store.save_outgoing_index(&Default::default(), 2, 1)?;

        let loaded = store.load_outgoing_index(&Default::default(), 1)?;
        assert_eq!(Some(1), loaded);

        let loaded = store.load_outgoing_index(&Default::default(), 2)?;
        assert_eq!(Some(1), loaded);
        Ok(())
    }

    pub fn out_index_should_update_existing_index_for_channel_epoch<S: OutgoingIndexStore>(mut store: S) -> anyhow::Result<()> {
        store.save_outgoing_index(&Default::default(), 1, 1)?;
        let loaded = store.load_outgoing_index(&Default::default(), 1)?;
        assert_eq!(Some(1), loaded);

        store.save_outgoing_index(&Default::default(), 1, 2)?;

        let loaded = store.load_outgoing_index(&Default::default(), 1)?;
        assert_eq!(Some(2), loaded);
        Ok(())
    }

    pub fn out_index_store_should_iterate_existing_indices_for_channel_epoch<S: OutgoingIndexStore>(mut store: S) -> anyhow::Result<()> {
        store.save_outgoing_index(&Default::default(), 1, 1)?;
        store.save_outgoing_index(&Default::default(), 2, 1)?;
        let loaded = store.load_outgoing_index(&Default::default(), 1)?;
        assert_eq!(Some(1), loaded);

        let loaded = store.load_outgoing_index(&Default::default(), 2)?;
        assert_eq!(Some(1), loaded);
        Ok(())
    }
}
