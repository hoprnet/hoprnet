use std::num::NonZeroU8;

use hopr_api::{
    HoprBalance,
    chain::{ChannelEntry, WinningProbability},
    tickets::TicketBuilder,
    types::{
        internal::channels::ChannelStatus,
        primitive::prelude::{U256, UnitaryFloatOps},
    },
};

use crate::{
    OutgoingIndexStore, TicketManagerError, TicketQueueStore,
    utils::{NoUnrealizedValue, OutgoingIndexCache, UnrealizedValue},
};

pub struct HoprTicketFactory<S> {
    out_idx_tracker: OutgoingIndexCache,
    queue_map: std::sync::Arc<dyn UnrealizedValue>,
    store: std::sync::Arc<parking_lot::RwLock<S>>,
}

impl<S: OutgoingIndexStore + 'static> HoprTicketFactory<S> {
    /// Creates a new independent ticket factory instance backed by the given `store`.
    pub fn new(store: S) -> Self {
        Self {
            out_idx_tracker: Default::default(),
            queue_map: std::sync::Arc::new(NoUnrealizedValue),
            store: std::sync::Arc::new(parking_lot::RwLock::new(store)),
        }
    }

    pub(crate) fn new_shared<Q: UnrealizedValue + 'static>(
        store: std::sync::Arc<parking_lot::RwLock<S>>,
        queue_map: std::sync::Arc<Q>,
    ) -> Self {
        Self {
            out_idx_tracker: Default::default(),
            queue_map,
            store,
        }
    }
}

impl<S> HoprTicketFactory<S>
where
    S: OutgoingIndexStore + Send + Sync + 'static,
{
    /// Gets the next usable ticket index for an outgoing ticket in the given channel and epoch.
    ///
    /// This operation is fast and does not immediately put the index into the [`OutgoingIndexStore`].
    ///
    /// The returned value is always guaranteed to be greater or equal to the ticket index on the given `channel`.
    fn next_outgoing_ticket_index(&self, channel: &ChannelEntry) -> u64 {
        let mut next_index = self.out_idx_tracker.next(channel.get_id(), channel.channel_epoch);
        tracing::trace!(%channel, next_index, "next outgoing ticket index");

        let epoch = channel.channel_epoch;

        if next_index < channel.ticket_index {
            // Correct the value in the cache if it was lower than the ticket index on the channel.
            // This sets the value in the cache to the next index after the ticket index on the channel.
            self.out_idx_tracker
                .upsert(channel.get_id(), epoch, channel.ticket_index + 1);
            next_index = channel.ticket_index; // Still, use the channel's ticket index as the next index.
        }

        // If this is the first index in this epoch,
        // remove the previous epoch from the map if any.
        // Epochs always start at 1, ticket indices at 0.
        if next_index == 0 && epoch > 1 && self.out_idx_tracker.remove(channel.get_id(), epoch - 1) {
            tracing::trace!(%channel, prev_epoch = epoch - 1, "removing previous epoch from outgoing index cache");
        }

        next_index
    }

    /// Saves outgoing ticket indices back to the store.
    ///
    /// The operation does nothing if there were no [new tickets created](HoprTicketManager::next_multihop_ticket)
    /// on any tracked channel.
    pub fn save_outgoing_indices(&self) -> Result<(), TicketManagerError> {
        self.out_idx_tracker
            .save(self.store.clone())
            .map_err(TicketManagerError::store)?;
        Ok(())
    }

    /// Synchronizes the outgoing index counters based on the current on-chain channel
    /// state given by `outgoing_channels`.
    ///
    /// Outgoing indices for channels that either are not present in `outgoing_channels` or
    /// not present as opened channels will be removed from the store.
    ///
    /// Outgoing indices for existing open channels in `outgoing_channels` will be either:
    /// - added to the store with their current index and epoch (if not present in the store), or
    /// - updated to the maximum of the two index values (if present in the store)
    ///
    /// It is advised to call this function early after the construction of the `HoprTicketManager`
    /// to ensure pruning of dangling or out-of-date values.
    pub fn sync_from_outgoing_channels(&self, outgoing_channels: &[ChannelEntry]) -> Result<(), TicketManagerError> {
        let outgoing_channels: std::collections::HashSet<_, std::hash::RandomState> =
            outgoing_channels.iter().collect();

        // Purge outdated outgoing indices
        let mut store_read = self.store.upgradable_read();
        let stored_indices = store_read
            .iter_outgoing_indices()
            .map_err(TicketManagerError::store)?
            .collect::<Vec<_>>();
        for (channel_id, epoch) in stored_indices {
            // If any stored outgoing index does not match any currently existing opened channel,
            // remove it from the store
            if !outgoing_channels.iter().any(|channel| {
                channel.status == ChannelStatus::Open
                    && channel.get_id() == &channel_id
                    && channel.channel_epoch == epoch
            }) {
                let mut store_write = parking_lot::RwLockUpgradableReadGuard::upgrade(store_read);
                store_write
                    .delete_outgoing_index(&channel_id, epoch)
                    .map_err(TicketManagerError::store)?;
                store_read = parking_lot::RwLockWriteGuard::downgrade_to_upgradable(store_write);
                tracing::debug!(%channel_id, epoch, "purging outdated outgoing index")
            }
        }

        for channel in outgoing_channels
            .iter()
            .filter(|channel| channel.status == ChannelStatus::Open)
        {
            let id = channel.get_id();

            // Either load a previously stored outgoing index or use the channel's ticket index as a
            // fallback
            let epoch = channel.channel_epoch;
            let index = match store_read.load_outgoing_index(id, epoch) {
                Ok(Some(out_index)) => out_index,
                Ok(None) => 0,
                Err(error) => {
                    tracing::error!(%error, %id, "failed to load outgoing index for channel, falling back to channel ticket index");
                    0
                }
            };

            // Always use the maximum from the stored value and the current ticket index on the channel
            let out_index = index.max(channel.ticket_index);
            self.out_idx_tracker.upsert(id, epoch, out_index);
            tracing::debug!(%id, epoch, out_index, "loaded outgoing ticket index for channel");
        }

        tracing::debug!(
            num_channels = outgoing_channels.len(),
            "synchronized with outgoing channels"
        );
        Ok(())
    }
}

impl<S> hopr_api::tickets::TicketFactory for HoprTicketFactory<S>
where
    S: OutgoingIndexStore + TicketQueueStore + Send + Sync + 'static,
    S::Queue: Send + Sync + 'static,
{
    type Error = TicketManagerError;

    /// Method fulfills the implementation of
    /// [`TicketFactory::new_multihop_ticket`](hopr_api::tickets::TicketFactory::new_multihop_ticket).
    ///
    /// ### Implementation details
    /// The `current_path_pos` indicates the position of the current hop in the multi-hop path.
    /// It is used to determine the value of the ticket: `price * (current_path_pos - 1) / winning_prob`.
    /// The function does not make sense for `current_path_pos <= 1` and returns an error if such an argument is
    /// provided.
    ///
    /// For last-hop tickets (`current_path_pos` equal to 1), a [zero hop ticket](TicketBuilder::zero_hop) should be
    /// created instead.
    ///
    /// The function will fail for channels that are not opened or do not have enough funds to cover the ticket value.
    /// The ticket index of the returned ticket is guaranteed to be greater or equal to the ticket index on the
    /// given `channel` argument.
    fn new_multihop_ticket(
        &self,
        channel: &ChannelEntry,
        path_position: NonZeroU8,
        winning_probability: WinningProbability,
        price_per_hop: HoprBalance,
    ) -> Result<TicketBuilder, Self::Error> {
        let current_path_pos = path_position.get();
        if current_path_pos == 1 {
            return Err(TicketManagerError::Other(anyhow::anyhow!(
                "current path position for multihop ticket must be greater than 1"
            )));
        }

        if channel.status != ChannelStatus::Open {
            return Err(TicketManagerError::Other(anyhow::anyhow!(
                "channel must be open to create a multihop ticket"
            )));
        }

        // The next ticket is worth: price * remaining hop count / winning probability
        let amount = HoprBalance::from(
            price_per_hop
                .amount()
                .saturating_mul(U256::from(current_path_pos - 1))
                .div_f64(winning_probability.into())
                .expect("winning probability is always less than or equal to 1"),
        );

        if channel.balance.lt(&amount) {
            return Err(TicketManagerError::OutOfFunds(*channel.get_id(), amount));
        }

        let ticket_builder = TicketBuilder::default()
            .counterparty(channel.destination)
            .balance(amount)
            .index(self.next_outgoing_ticket_index(channel))
            .win_prob(winning_probability)
            .channel_epoch(channel.channel_epoch);

        Ok(ticket_builder)
    }

    fn remaining_incoming_channel_stake(&self, channel: &ChannelEntry) -> Result<HoprBalance, Self::Error> {
        let unrealized_value = self
            .queue_map
            .unrealized_value(channel.get_id(), channel.ticket_index.into())?;

        // Subtraction on HoprBalance type naturally saturating at 0
        Ok(channel.balance - unrealized_value.unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use hopr_api::{tickets::TicketFactory, types::crypto::prelude::Keypair};
    use hopr_chain_connector::ChainKeypair;

    use super::*;
    use crate::MemoryStore;

    fn create_mgr() -> anyhow::Result<HoprTicketFactory<MemoryStore>> {
        Ok(HoprTicketFactory::new(MemoryStore::default()))
    }

    #[test]
    fn ticket_factory_should_create_multihop_tickets() -> anyhow::Result<()> {
        let mgr = create_mgr()?;

        let src = ChainKeypair::random();
        let dst = ChainKeypair::random();

        let channel = ChannelEntry::builder()
            .between(&src, &dst)
            .amount(10)
            .ticket_index(1)
            .status(ChannelStatus::Open)
            .epoch(1)
            .build()?;

        // Loads index 1 which is the next index for a ticket on this channel
        mgr.sync_from_outgoing_channels(&[channel])?;

        let ticket_1 = mgr
            .new_multihop_ticket(&channel, 2.try_into()?, WinningProbability::ALWAYS, 10.into())?
            .eth_challenge(Default::default())
            .build_signed(&src, &Default::default())?;

        assert_eq!(ticket_1.channel_id(), channel.get_id());
        assert_eq!(channel.ticket_index, ticket_1.verified_ticket().index);
        assert_eq!(channel.channel_epoch, ticket_1.verified_ticket().channel_epoch);

        let ticket_2 = mgr
            .new_multihop_ticket(&channel, 2.try_into()?, WinningProbability::ALWAYS, 10.into())?
            .eth_challenge(Default::default())
            .build_signed(&src, &Default::default())?;

        assert_eq!(ticket_2.channel_id(), channel.get_id());
        assert_eq!(channel.ticket_index + 1, ticket_2.verified_ticket().index);
        assert_eq!(channel.channel_epoch, ticket_2.verified_ticket().channel_epoch);

        // Should not accept path positions less than 2
        assert!(
            mgr.new_multihop_ticket(&channel, 1.try_into()?, WinningProbability::ALWAYS, 10.into())
                .is_err()
        );

        Ok(())
    }

    #[test]
    fn ticket_manager_create_multihop_ticket_should_fail_on_wrong_input() -> anyhow::Result<()> {
        let mgr = create_mgr()?;

        let src = ChainKeypair::random();
        let dst = ChainKeypair::random();

        let mut channel = ChannelEntry::builder()
            .between(&src, &dst)
            .amount(10)
            .ticket_index(1)
            .status(ChannelStatus::Closed)
            .epoch(1)
            .build()?;

        assert!(
            mgr.new_multihop_ticket(&channel, 2.try_into()?, WinningProbability::ALWAYS, 1.into())
                .is_err()
        );

        channel.status =
            ChannelStatus::PendingToClose(std::time::SystemTime::now() - std::time::Duration::from_secs(10));

        assert!(
            mgr.new_multihop_ticket(&channel, 2.try_into()?, WinningProbability::ALWAYS, 1.into())
                .is_err()
        );

        channel.status = ChannelStatus::Open;

        assert!(
            mgr.new_multihop_ticket(&channel, 2.try_into()?, WinningProbability::ALWAYS, 11.into())
                .is_err()
        );

        assert!(
            mgr.new_multihop_ticket(&channel, 1.try_into()?, WinningProbability::ALWAYS, 1.into())
                .is_err()
        );

        Ok(())
    }

    #[test]
    fn ticket_manager_test_next_outgoing_ticket_index() -> anyhow::Result<()> {
        let mgr = create_mgr()?;

        let src = ChainKeypair::random();
        let dst = ChainKeypair::random();

        let mut channel = ChannelEntry::builder()
            .between(&src, &dst)
            .amount(10)
            .ticket_index(0)
            .status(ChannelStatus::Open)
            .epoch(1)
            .build()?;

        assert_eq!(0, mgr.next_outgoing_ticket_index(&channel));

        channel.ticket_index = 10;
        assert_eq!(10, mgr.next_outgoing_ticket_index(&channel));
        assert_eq!(11, mgr.next_outgoing_ticket_index(&channel));

        channel.ticket_index = 100;
        assert_eq!(100, mgr.next_outgoing_ticket_index(&channel));
        assert_eq!(101, mgr.next_outgoing_ticket_index(&channel));

        channel.ticket_index = 50;
        assert_eq!(102, mgr.next_outgoing_ticket_index(&channel));
        assert_eq!(103, mgr.next_outgoing_ticket_index(&channel));

        mgr.save_outgoing_indices()?;
        assert_eq!(Some(104), mgr.store.read().load_outgoing_index(channel.get_id(), 1)?);

        channel.ticket_index = 0;
        channel.channel_epoch = 2;

        assert_eq!(0, mgr.next_outgoing_ticket_index(&channel));
        mgr.save_outgoing_indices()?;

        assert_eq!(None, mgr.store.read().load_outgoing_index(channel.get_id(), 1)?);
        assert_eq!(Some(1), mgr.store.read().load_outgoing_index(channel.get_id(), 2)?);

        assert_eq!(1, mgr.next_outgoing_ticket_index(&channel));

        Ok(())
    }

    #[test]
    fn ticket_manager_should_save_out_indices_to_the_store_on_demand() -> anyhow::Result<()> {
        let mgr = create_mgr()?;

        let src = ChainKeypair::random();
        let dst = ChainKeypair::random();

        let channel = ChannelEntry::builder()
            .between(&src, &dst)
            .amount(10)
            .ticket_index(1)
            .status(ChannelStatus::Open)
            .epoch(1)
            .build()?;

        // Loads index 1 which is the next index for a ticket on this channel
        mgr.sync_from_outgoing_channels(&[channel])?;

        mgr.new_multihop_ticket(&channel, 2.try_into()?, WinningProbability::ALWAYS, 10.into())?;

        // Without saving, the store index should not be present in store
        let saved_index = mgr.store.read().load_outgoing_index(channel.get_id(), 1)?;
        assert_eq!(None, saved_index);

        mgr.new_multihop_ticket(&channel, 2.try_into()?, WinningProbability::ALWAYS, 10.into())?;

        mgr.save_outgoing_indices()?;
        let saved_index = mgr.store.read().load_outgoing_index(channel.get_id(), 1)?;
        assert_eq!(Some(3), saved_index);

        mgr.new_multihop_ticket(&channel, 2.try_into()?, WinningProbability::ALWAYS, 10.into())?;

        let saved_index = mgr.store.read().load_outgoing_index(channel.get_id(), 1)?;
        assert_eq!(Some(3), saved_index);

        mgr.save_outgoing_indices()?;
        let saved_index = mgr.store.read().load_outgoing_index(channel.get_id(), 1)?;
        assert_eq!(Some(4), saved_index);

        Ok(())
    }

    #[test]
    fn ticket_manager_should_sync_out_indices_from_chain_state() -> anyhow::Result<()> {
        let mgr = create_mgr()?;

        let src = ChainKeypair::random();
        let dst = ChainKeypair::random();

        let channel = ChannelEntry::builder()
            .between(&src, &dst)
            .amount(10)
            .ticket_index(1)
            .status(ChannelStatus::Open)
            .epoch(1)
            .build()?;

        mgr.sync_from_outgoing_channels(&[channel])?;
        mgr.save_outgoing_indices()?;

        let saved_index = mgr.store.read().load_outgoing_index(channel.get_id(), 1)?;
        assert_eq!(Some(1), saved_index);

        Ok(())
    }

    #[test_log::test]
    fn ticket_manager_should_sync_out_indices_should_remove_indices_for_non_opened_outgoing_channels()
    -> anyhow::Result<()> {
        let mgr = create_mgr()?;

        let src = ChainKeypair::random();
        let dst = ChainKeypair::random();

        let mut channel_1 = ChannelEntry::builder()
            .between(&src, &dst)
            .amount(10)
            .ticket_index(0)
            .status(ChannelStatus::Open)
            .epoch(1)
            .build()?;

        let mut channel_2 = ChannelEntry::builder()
            .between(&dst, &src)
            .amount(10)
            .ticket_index(0)
            .status(ChannelStatus::Open)
            .epoch(1)
            .build()?;

        let ticket_1 = mgr
            .new_multihop_ticket(&channel_1, 2.try_into()?, WinningProbability::ALWAYS, 10.into())?
            .eth_challenge(Default::default())
            .build()?;
        let ticket_2 = mgr
            .new_multihop_ticket(&channel_2, 2.try_into()?, WinningProbability::ALWAYS, 10.into())?
            .eth_challenge(Default::default())
            .build()?;
        assert_eq!(0, ticket_1.index);
        assert_eq!(0, ticket_2.index);

        mgr.save_outgoing_indices()?;

        assert_eq!(Some(1), mgr.store.read().load_outgoing_index(channel_1.get_id(), 1)?);
        assert_eq!(Some(1), mgr.store.read().load_outgoing_index(channel_2.get_id(), 1)?);

        channel_1.status = ChannelStatus::Closed;
        channel_2.status =
            ChannelStatus::PendingToClose(std::time::SystemTime::now() - std::time::Duration::from_mins(10));

        mgr.sync_from_outgoing_channels(&[channel_1, channel_2])?;

        assert_eq!(None, mgr.store.read().load_outgoing_index(channel_1.get_id(), 1)?);
        assert_eq!(None, mgr.store.read().load_outgoing_index(channel_2.get_id(), 1)?);

        Ok(())
    }
}
