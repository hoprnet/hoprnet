use hopr_api::{
    chain::{ChannelId, HoprBalance, RedeemableTicket},
    types::internal::prelude::Ticket,
};

/// Allows loading and saving outgoing ticket indices.
pub trait OutgoingIndexStore {
    type Error: std::error::Error + Send + Sync + 'static;
    /// Loads the last used outgoing ticket index for the given channel and epoch.
    ///
    /// If the index is not found, returns `Ok(None)`.
    fn load_outgoing_index(&self, channel_id: &ChannelId, epoch: u32) -> Result<Option<u64>, Self::Error>;
    /// Saves the last used outgoing ticket index for the given channel and epoch.
    fn save_outgoing_index(&mut self, channel_id: &ChannelId, epoch: u32, index: u64) -> Result<(), Self::Error>;
    /// Deletes the outgoing ticket index for the given channel and epoch.
    fn delete_outgoing_index(&mut self, channel_id: &ChannelId, epoch: u32) -> Result<(), Self::Error>;
    /// Iterate over all channel IDs and epochs of outgoing ticket indices in the storage.
    fn iter_outgoing_indices(&self) -> Result<impl Iterator<Item = (ChannelId, u32)>, Self::Error>;
}

/// Allows loading ticket queues from a storage.
pub trait TicketQueueStore {
    /// Type of queues.
    type Queue: TicketQueue;
    /// Opens or creates a new queue in storage for the given channel.
    fn open_or_create_queue(
        &mut self,
        channel_id: &ChannelId,
    ) -> Result<Self::Queue, <Self::Queue as TicketQueue>::Error>;
    /// Deletes the queue for the given channel.
    ///
    /// Returns any tickets left-over in this queue if it existed.
    /// Returned tickets are no longer redeemable.
    fn delete_queue(&mut self, channel_id: &ChannelId) -> Result<Vec<Ticket>, <Self::Queue as TicketQueue>::Error>;
    /// Iterate over all channel IDs of ticket queues in the storage.
    fn iter_queues(&self) -> Result<impl Iterator<Item = ChannelId>, <Self::Queue as TicketQueue>::Error>;
}

/// Backend for ticket storage queue.
///
/// The implementations must honor the natural ordering of tickets.
pub trait TicketQueue {
    type Error: std::error::Error + Send + Sync + 'static;
    /// Number of tickets in the queue.
    fn len(&self) -> Result<usize, Self::Error>;
    /// Indicates whether the queue is empty.
    fn is_empty(&self) -> Result<bool, Self::Error> {
        Ok(self.len()? == 0)
    }
    /// Add a ticket to the queue.
    fn push(&mut self, ticket: RedeemableTicket) -> Result<(), Self::Error>;
    /// Remove and return the next ticket in-order from the queue.
    fn pop(&mut self) -> Result<Option<RedeemableTicket>, Self::Error>;
    /// Return the next ticket in-order from the queue without removing it.
    fn peek(&self) -> Result<Option<RedeemableTicket>, Self::Error>;
    /// Iterate over all tickets in the queue in **arbitrary** order.
    fn iter_unordered(&self) -> Result<impl Iterator<Item = Result<RedeemableTicket, Self::Error>>, Self::Error>;
    /// Computes the total value of tickets of the given epoch (and optionally minimum given index)
    /// in this queue.
    ///
    /// The default implementation simply [iterates](TicketQueue::iter_unordered) the queue
    /// and sums the total value of matching tickets.
    fn total_value(&self, epoch: u32, min_index: Option<u64>) -> Result<HoprBalance, Self::Error> {
        default_total_value(self, epoch, min_index)
    }
    /// Drains all the remaining tickets from the queue, rendering them no longer redeemable.
    ///
    /// The drained tickets are still ordered according to their natural ordering.
    fn drain(&mut self) -> Result<Vec<Ticket>, Self::Error> {
        let mut tickets = Vec::new();
        while let Some(ticket) = self.pop()? {
            tickets.push(*ticket.verified_ticket());
        }
        Ok(tickets)
    }
}

pub(crate) fn default_total_value<Q: TicketQueue + ?Sized>(
    queue: &Q,
    epoch: u32,
    min_index: Option<u64>,
) -> Result<HoprBalance, Q::Error> {
    let min_index = min_index.unwrap_or(0);
    Ok(queue
        .iter_unordered()?
        .filter_map(|res| {
            res.ok()
                .filter(|t| t.verified_ticket().channel_epoch == epoch && t.verified_ticket().index >= min_index)
                .map(|t| t.verified_ticket().amount)
        })
        .sum())
}

#[cfg(test)]
pub(crate) mod tests {
    use hopr_api::{
        chain::{ChannelId, HoprBalance, RedeemableTicket, WinningProbability},
        types::{crypto::prelude::*, crypto_random::Randomizable, internal::prelude::TicketBuilder},
    };
    use rand::prelude::SliceRandom;

    use crate::{OutgoingIndexStore, TicketQueue, TicketQueueStore};

    const TICKET_VALUE: u64 = 10;

    pub fn generate_tickets() -> anyhow::Result<Vec<RedeemableTicket>> {
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

    pub fn fill_queue<Q: TicketQueue, I: Iterator<Item = RedeemableTicket>>(
        queue: &mut Q,
        iter: I,
    ) -> anyhow::Result<()> {
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

    pub fn queue_returns_all_tickets<Q: TicketQueue>(mut queue: Q) -> anyhow::Result<()> {
        let mut tickets = generate_tickets()?;
        tickets.sort();

        fill_queue(&mut queue, tickets.iter().copied())?;

        let mut collected_tickets = queue.iter_unordered()?.filter_map(|r| r.ok()).collect::<Vec<_>>();
        collected_tickets.sort();

        assert_eq!(tickets, collected_tickets);
        Ok(())
    }

    pub fn queue_is_empty_when_drained<Q: TicketQueue>(mut queue: Q) -> anyhow::Result<()> {
        fill_queue(&mut queue, generate_tickets()?.into_iter())?;
        assert!(!queue.is_empty()?);

        while let Some(_) = queue.pop()? {}
        assert!(queue.is_empty()?);
        Ok(())
    }

    pub fn queue_returns_empty_iterator_when_drained<Q: TicketQueue>(mut queue: Q) -> anyhow::Result<()> {
        fill_queue(&mut queue, generate_tickets()?.into_iter())?;
        assert!(!queue.is_empty()?);

        while let Some(_) = queue.pop()? {}
        assert_eq!(queue.iter_unordered()?.count(), 0);
        Ok(())
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

    pub fn ticket_store_should_create_new_queue_for_channel<S: TicketQueueStore>(mut store: S) -> anyhow::Result<()> {
        assert_eq!(0, store.iter_queues()?.count());

        let _ = store.open_or_create_queue(&Default::default())?;
        let queues = store.iter_queues()?.collect::<Vec<_>>();
        assert_eq!(1, queues.len());
        assert!(queues.contains(&Default::default()));

        Ok(())
    }

    pub fn ticket_store_should_open_existing_queue_for_channel<S: TicketQueueStore>(
        mut store: S,
    ) -> anyhow::Result<()> {
        assert_eq!(0, store.iter_queues()?.count());

        let mut tickets = generate_tickets()?;
        let mut queue = store.open_or_create_queue(&Default::default())?;
        fill_queue(&mut queue, tickets.iter().copied())?;
        tickets.sort();

        let queue = store.open_or_create_queue(&Default::default())?;
        let opened_tickets = queue.iter_unordered()?.filter_map(|r| r.ok()).collect::<Vec<_>>();

        assert_eq!(tickets, opened_tickets);

        Ok(())
    }

    pub fn ticket_store_should_delete_existing_queue_for_channel<S: TicketQueueStore>(
        mut store: S,
    ) -> anyhow::Result<()> {
        assert_eq!(0, store.iter_queues()?.count());

        let _ = store.open_or_create_queue(&Default::default())?;
        let queues = store.iter_queues()?.collect::<Vec<_>>();
        assert_eq!(1, queues.len());
        assert!(queues.contains(&Default::default()));

        let _ = store.delete_queue(&Default::default())?;
        let queues = store.iter_queues()?.collect::<Vec<_>>();
        assert_eq!(0, queues.len());

        Ok(())
    }

    pub fn ticket_store_should_iterate_existing_queues_for_channel<S: TicketQueueStore>(
        mut store: S,
    ) -> anyhow::Result<()> {
        assert_eq!(0, store.iter_queues()?.count());

        let c1 = ChannelId::create(&[b"beef"]);
        let _ = store.open_or_create_queue(&c1)?;
        let queues = store.iter_queues()?.collect::<Vec<_>>();
        assert_eq!(1, queues.len());
        assert!(queues.contains(&c1));

        let c2 = ChannelId::create(&[b"feed"]);
        let _ = store.open_or_create_queue(&c2)?;
        let queues = store.iter_queues()?.collect::<Vec<_>>();

        assert_eq!(2, queues.len());
        assert!(queues.contains(&c1));
        assert!(queues.contains(&c2));

        Ok(())
    }

    pub fn ticket_store_should_not_fail_to_delete_non_existing_queue_for_channel<S: TicketQueueStore>(
        mut store: S,
    ) -> anyhow::Result<()> {
        assert_eq!(0, store.iter_queues()?.count());
        assert!(store.delete_queue(&Default::default()).is_ok());

        Ok(())
    }

    pub fn out_index_store_should_load_existing_index_for_channel_epoch<S: OutgoingIndexStore>(
        mut store: S,
    ) -> anyhow::Result<()> {
        store.save_outgoing_index(&Default::default(), 1, 1)?;
        let loaded = store.load_outgoing_index(&Default::default(), 1)?;
        assert_eq!(Some(1), loaded);
        Ok(())
    }

    pub fn out_index_store_should_not_load_non_existing_index_for_channel_epoch<S: OutgoingIndexStore>(
        store: S,
    ) -> anyhow::Result<()> {
        let loaded = store.load_outgoing_index(&Default::default(), 1)?;
        assert_eq!(None, loaded);
        Ok(())
    }

    pub fn out_index_store_should_delete_existing_index_for_channel_epoch<S: OutgoingIndexStore>(
        mut store: S,
    ) -> anyhow::Result<()> {
        store.save_outgoing_index(&Default::default(), 1, 1)?;
        let loaded = store.load_outgoing_index(&Default::default(), 1)?;
        assert_eq!(Some(1), loaded);
        store.delete_outgoing_index(&Default::default(), 1)?;
        let loaded = store.load_outgoing_index(&Default::default(), 1)?;
        assert_eq!(None, loaded);
        Ok(())
    }

    pub fn out_index_store_should_store_new_index_for_channel_epoch<S: OutgoingIndexStore>(
        mut store: S,
    ) -> anyhow::Result<()> {
        store.save_outgoing_index(&Default::default(), 1, 1)?;
        store.save_outgoing_index(&Default::default(), 2, 1)?;

        let loaded = store.load_outgoing_index(&Default::default(), 1)?;
        assert_eq!(Some(1), loaded);

        let loaded = store.load_outgoing_index(&Default::default(), 2)?;
        assert_eq!(Some(1), loaded);
        Ok(())
    }

    pub fn out_index_should_update_existing_index_for_channel_epoch<S: OutgoingIndexStore>(
        mut store: S,
    ) -> anyhow::Result<()> {
        store.save_outgoing_index(&Default::default(), 1, 1)?;
        let loaded = store.load_outgoing_index(&Default::default(), 1)?;
        assert_eq!(Some(1), loaded);

        store.save_outgoing_index(&Default::default(), 1, 2)?;

        let loaded = store.load_outgoing_index(&Default::default(), 1)?;
        assert_eq!(Some(2), loaded);
        Ok(())
    }

    pub fn out_index_store_should_iterate_existing_indices_for_channel_epoch<S: OutgoingIndexStore>(
        mut store: S,
    ) -> anyhow::Result<()> {
        store.save_outgoing_index(&Default::default(), 1, 1)?;
        store.save_outgoing_index(&Default::default(), 2, 1)?;
        let loaded = store.load_outgoing_index(&Default::default(), 1)?;
        assert_eq!(Some(1), loaded);

        let loaded = store.load_outgoing_index(&Default::default(), 2)?;
        assert_eq!(Some(1), loaded);
        Ok(())
    }
}
