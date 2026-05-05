//! Five-pass decision pipeline: snapshot → fund → close → finalize → open.
//!
//! Also contains the helper utilities and action dispatchers (`try_*` methods)
//! that the pipeline calls to submit chain transactions.

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::{Duration, Instant},
};

use futures::StreamExt as _;
use hopr_lib::api::{
    PeerId,
    chain::{
        AccountSelector, ChainReadAccountOperations, ChainReadChannelOperations, ChainReadSafeOperations, ChainValues,
        ChainWriteChannelOperations, ChannelSelector, SafeSelector,
    },
    graph::{EdgeObservableRead as _, NetworkGraphView as _},
    network::NetworkView as _,
    node::{ActionableEventSource, HasChainApi, HasGraphView, HasNetworkView},
    types::{
        crypto::prelude::OffchainPublicKey,
        internal::prelude::{ChannelEntry, ChannelStatus},
        primitive::prelude::{Address, HoprBalance},
    },
};
use tracing::{debug, info, warn};

use crate::errors::StrategyError;

use super::{ChannelLifecycleStrategyInner, ChannelObservation};

impl<N> ChannelLifecycleStrategyInner<N>
where
    N: HasChainApi + HasNetworkView + HasGraphView + ActionableEventSource + Send + Sync + 'static,
    N::ChainApi: ChainReadChannelOperations
        + ChainReadSafeOperations
        + ChainReadAccountOperations
        + ChainValues
        + ChainWriteChannelOperations
        + Clone
        + Send
        + Sync
        + 'static,
{
    // ─────────────────────────────────────────────────────────────────────
    // Helpers
    // ─────────────────────────────────────────────────────────────────────

    /// Returns the available safe balance, or `HoprBalance::zero()` if the safe
    /// is not registered.
    pub(super) async fn safe_balance_budget(&self) -> crate::errors::Result<HoprBalance> {
        let me = *self.node.chain_api().me();
        let chain = self.node.chain_api().clone();

        let safe = chain
            .safe_info(SafeSelector::NodeAddress(me))
            .await
            .map_err(|e| StrategyError::Other(e.into()))?;

        let Some(safe) = safe else {
            warn!(%me, "channel-lifecycle: safe not registered");
            return Ok(HoprBalance::zero());
        };

        chain
            .balance(safe.address)
            .await
            .map_err(|e| StrategyError::Other(e.into()))
    }

    /// Returns the chain's estimated transaction confirmation time, falling
    /// back to the configured default on error.
    pub(super) async fn est_tx_time(&self) -> Duration {
        match self.node.chain_api().typical_resolution_time().await {
            Ok(d) => d,
            Err(e) => {
                debug!(%e, "channel-lifecycle: typical_resolution_time failed, using fallback");
                self.cfg.proactive_funding.fallback_chain_op_duration
            }
        }
    }

    /// Returns the on-chain notice period for channel closure, falling back
    /// to a conservative 5-minute default on error.
    async fn closure_notice_period(&self) -> Duration {
        match self.node.chain_api().channel_closure_notice_period().await {
            Ok(d) => d,
            Err(e) => {
                warn!(%e, "channel-lifecycle: could not fetch channel_closure_notice_period");
                Duration::from_secs(5 * 60)
            }
        }
    }

    /// Total number of in-flight chain-write operations across all four kinds.
    fn total_in_flight(&self) -> usize {
        self.open_in_flight.len()
            + self.fund_in_flight.len()
            + self.close_in_flight.len()
            + self.finalize_in_flight.len()
    }

    /// Composite quality score for a peer, blending the graph edge score with
    /// a normalised ticket-activity signal.
    fn peer_score_for(&self, peer_offchain: &OffchainPublicKey, dest_addr: &Address) -> f64 {
        let my_key = self.node.graph().identity();
        let edge_score = self
            .node
            .graph()
            .edge(my_key, peer_offchain)
            .map(|e| e.score())
            .unwrap_or(0.0);

        let ticket_delta = self.peer_ticket_activity.get(dest_addr).map(|v| *v).unwrap_or(0);
        let max_activity = self
            .peer_ticket_activity
            .iter()
            .map(|e| *e.value())
            .max()
            .unwrap_or(1)
            .max(1);
        let ticket_score = (ticket_delta as f64) / (max_activity as f64);

        self.cfg.eligibility.peer_quality_weight * edge_score
            + self.cfg.eligibility.ticket_activity_weight * ticket_score
    }

    /// Returns `true` if the channel should be proactively funded before the
    /// next transaction confirms.
    fn proactive_fund_needed(&self, channel: &ChannelEntry, est_tx_secs: f64, min_ticket_price_wei: f64) -> bool {
        if !self.cfg.proactive_funding.enabled {
            return false;
        }

        let obs = match self.last_observed.get(channel.get_id()) {
            Some(o) => o.clone(),
            None => return false,
        };

        let lookback_secs = self.cfg.proactive_funding.depletion_lookback.as_secs_f64().max(1.0);
        let elapsed = obs.at.elapsed().as_secs_f64().max(0.01);

        let balance_now = obs.balance.amount().low_u128() as f64;
        let balance_then = channel.balance.amount().low_u128() as f64;
        let balance_delta = (balance_then - balance_now).max(0.0);
        let balance_drain_rate =
            (self.cfg.proactive_funding.balance_drain_weight * balance_delta / elapsed.min(lookback_secs)).max(0.0);

        let ticket_delta = channel.ticket_index.saturating_sub(obs.ticket_index) as f64;
        let ticket_drain_rate =
            (self.cfg.proactive_funding.ticket_index_drain_weight * ticket_delta * min_ticket_price_wei
                / elapsed.min(lookback_secs))
            .max(0.0);

        let drain_rate = balance_drain_rate + ticket_drain_rate;
        let projected_drain = drain_rate * est_tx_secs * self.cfg.proactive_funding.safety_margin;

        let balance_after = channel.balance.amount().low_u128() as f64 - projected_drain;
        let threshold = self.cfg.funding.lower_balance_threshold.amount().low_u128() as f64;

        balance_after < threshold
    }

    // ─────────────────────────────────────────────────────────────────────
    // Action dispatchers
    // ─────────────────────────────────────────────────────────────────────

    /// Spawn a funding transaction for `channel`.  Returns `true` if the task
    /// was submitted; `false` if it was already in-flight or the concurrency
    /// cap was reached.
    pub(super) fn try_fund_channel(&self, channel: &ChannelEntry, topup: HoprBalance) -> bool {
        let channel_id = *channel.get_id();

        if !self.fund_in_flight.insert(channel_id) {
            return false;
        }
        if self.total_in_flight() > self.cfg.concurrency.max_concurrent_actions {
            self.fund_in_flight.remove(&channel_id);
            return false;
        }

        info!(%channel, %topup, "channel-lifecycle: funding channel");
        #[cfg(all(feature = "telemetry", not(test)))]
        super::METRIC_CHANNEL_FUNDS.increment();

        let chain = self.node.chain_api().clone();
        let in_flight = Arc::clone(&self.fund_in_flight);

        hopr_async_runtime::prelude::spawn(async move {
            match chain.fund_channel(&channel_id, topup).await {
                Ok(confirmation) => {
                    if let Err(e) = confirmation.await {
                        warn!(%channel_id, %e, "channel-lifecycle: funding tx failed");
                        in_flight.remove(&channel_id);
                    }
                    // On success: ChannelBalanceIncreased event clears in_flight.
                }
                Err(e) => {
                    warn!(%channel_id, %e, "channel-lifecycle: failed to submit funding tx");
                    in_flight.remove(&channel_id);
                }
            }
        });

        true
    }

    /// Spawn a closure transaction for `channel`.  Returns `true` if submitted.
    fn try_close_channel(&self, channel: &ChannelEntry) -> bool {
        let channel_id = *channel.get_id();

        if !self.close_in_flight.insert(channel_id) {
            return false;
        }
        if self.total_in_flight() > self.cfg.concurrency.max_concurrent_actions {
            self.close_in_flight.remove(&channel_id);
            return false;
        }

        info!(%channel, "channel-lifecycle: closing channel");
        #[cfg(all(feature = "telemetry", not(test)))]
        super::METRIC_CHANNEL_CLOSES.increment();

        let chain = self.node.chain_api().clone();
        let in_flight = Arc::clone(&self.close_in_flight);

        hopr_async_runtime::prelude::spawn(async move {
            match chain.close_channel(&channel_id).await {
                Ok(confirmation) => {
                    if let Err(e) = confirmation.await {
                        warn!(%channel_id, %e, "channel-lifecycle: close tx failed");
                        in_flight.remove(&channel_id);
                    }
                    // On success: ChannelClosureInitiated event clears in_flight.
                }
                Err(e) => {
                    warn!(%channel_id, %e, "channel-lifecycle: failed to submit close tx");
                    in_flight.remove(&channel_id);
                }
            }
        });

        true
    }

    /// Spawn a finalization transaction for a `PendingToClose` channel.
    /// Returns `true` if submitted.
    fn try_finalize_channel(&self, channel: &ChannelEntry) -> bool {
        let channel_id = *channel.get_id();

        if !self.finalize_in_flight.insert(channel_id) {
            return false;
        }
        if self.total_in_flight() > self.cfg.concurrency.max_concurrent_actions {
            self.finalize_in_flight.remove(&channel_id);
            return false;
        }

        info!(%channel, "channel-lifecycle: finalizing closure");
        #[cfg(all(feature = "telemetry", not(test)))]
        super::METRIC_CHANNEL_FINALIZES.increment();

        let chain = self.node.chain_api().clone();
        let in_flight = Arc::clone(&self.finalize_in_flight);

        hopr_async_runtime::prelude::spawn(async move {
            match chain.close_channel(&channel_id).await {
                Ok(confirmation) => {
                    if let Err(e) = confirmation.await {
                        warn!(%channel_id, %e, "channel-lifecycle: finalize tx failed");
                        in_flight.remove(&channel_id);
                    }
                    // On success: ChannelClosed event clears in_flight.
                }
                Err(e) => {
                    warn!(%channel_id, %e, "channel-lifecycle: failed to submit finalize tx");
                    in_flight.remove(&channel_id);
                }
            }
        });

        true
    }

    /// Spawn an open transaction for a new channel to `dest`.  Returns `true`
    /// if submitted.
    fn try_open_channel(&self, dest: Address, amount: HoprBalance) -> bool {
        if !self.open_in_flight.insert(dest) {
            return false;
        }
        if self.total_in_flight() > self.cfg.concurrency.max_concurrent_actions {
            self.open_in_flight.remove(&dest);
            return false;
        }

        info!(%dest, %amount, "channel-lifecycle: opening channel");
        #[cfg(all(feature = "telemetry", not(test)))]
        super::METRIC_CHANNEL_OPENS.increment();

        let chain = self.node.chain_api().clone();
        let in_flight = Arc::clone(&self.open_in_flight);

        hopr_async_runtime::prelude::spawn(async move {
            match chain.open_channel(&dest, amount).await {
                Ok(confirmation) => {
                    if let Err(e) = confirmation.await {
                        warn!(%dest, %e, "channel-lifecycle: open tx failed");
                        in_flight.remove(&dest);
                    }
                    // On success: ChannelOpened event clears in_flight.
                }
                Err(e) => {
                    warn!(%dest, %e, "channel-lifecycle: failed to submit open tx");
                    in_flight.remove(&dest);
                }
            }
        });

        true
    }

    // ─────────────────────────────────────────────────────────────────────
    // Pipeline
    // ─────────────────────────────────────────────────────────────────────

    pub(super) async fn run_pipeline(&self) {
        if let Err(e) = self.pipeline_inner().await {
            warn!(%e, "channel-lifecycle: pipeline error");
        }
    }

    async fn pipeline_inner(&self) -> crate::errors::Result<()> {
        let chain = self.node.chain_api();
        let me = *chain.me();

        // ── 1. Snapshot ──────────────────────────────────────────────────────
        let est_tx_secs = self.est_tx_time().await.as_secs_f64();
        let safe_balance = self.safe_balance_budget().await?;

        let mut all_channels: Vec<ChannelEntry> = Vec::new();
        {
            let mut s = chain
                .stream_channels(ChannelSelector::default().with_source(me))
                .map_err(|e| StrategyError::Other(e.into()))?;
            while let Some(ch) = s.next().await {
                all_channels.push(ch);
            }
        }

        for ch in &all_channels {
            let id = *ch.get_id();
            self.last_observed
                .entry(id)
                .and_modify(|obs| {
                    if obs.balance != ch.balance || obs.ticket_index != ch.ticket_index {
                        *obs = ChannelObservation {
                            balance: ch.balance,
                            ticket_index: ch.ticket_index,
                            at: Instant::now(),
                        };
                    }
                })
                .or_insert_with(|| ChannelObservation {
                    balance: ch.balance,
                    ticket_index: ch.ticket_index,
                    at: Instant::now(),
                });
        }

        let min_ticket_price_wei = chain
            .minimum_ticket_price()
            .await
            .map(|p| p.amount().low_u128() as f64)
            .unwrap_or(0.0);

        let mut peer_addr_map: HashMap<PeerId, (OffchainPublicKey, Address)> = HashMap::new();
        {
            let mut accounts = chain
                .stream_accounts(AccountSelector::default())
                .map_err(|e| StrategyError::Other(e.into()))?;
            while let Some(account) = accounts.next().await {
                let peer_id = PeerId::from(&account.public_key);
                peer_addr_map.insert(peer_id, (account.public_key, account.chain_addr));
            }
        }

        let addr_to_key: HashMap<Address, OffchainPublicKey> =
            peer_addr_map.values().map(|(pk, addr)| (*addr, *pk)).collect();

        let open_channels: Vec<&ChannelEntry> = all_channels
            .iter()
            .filter(|c| c.status == ChannelStatus::Open)
            .collect();
        let open_count = open_channels.len() + self.open_in_flight.len();

        // ── 2. Fund pass ─────────────────────────────────────────────────────
        if safe_balance >= self.cfg.funding.min_safe_balance_required || !self.cfg.funding.stop_when_unfunded {
            let mut safe_remaining = safe_balance;

            for ch in &open_channels {
                if self.fund_in_flight.contains(ch.get_id()) {
                    continue;
                }
                let needs_topup = ch.balance <= self.cfg.funding.lower_balance_threshold;
                let needs_proactive = self.proactive_fund_needed(ch, est_tx_secs, min_ticket_price_wei);

                if needs_topup || needs_proactive {
                    if safe_remaining < self.cfg.funding.topup_balance {
                        debug!("channel-lifecycle: safe balance exhausted in fund pass");
                        break;
                    }
                    if self.try_fund_channel(ch, self.cfg.funding.topup_balance) {
                        safe_remaining -= self.cfg.funding.topup_balance;
                    }
                }
            }
        }

        // ── 3. Close pass ─────────────────────────────────────────────────────
        if self.start_epoch.elapsed() >= self.cfg.restart.startup_close_grace_period {
            let mut close_count = self.close_in_flight.len();

            for ch in &open_channels {
                if close_count >= self.cfg.closure.close_max_concurrent {
                    break;
                }
                if self.close_in_flight.contains(ch.get_id()) {
                    continue;
                }
                let remaining_open = open_count.saturating_sub(close_count);
                if remaining_open <= self.cfg.population.min_open_channels {
                    break;
                }

                if self.should_close(ch, &addr_to_key) && self.try_close_channel(ch) {
                    close_count += 1;
                }
            }
        }

        // ── 4. Finalize pass ──────────────────────────────────────────────────
        if self.cfg.finalizer.enabled {
            let overdue = self.closure_notice_period().await + self.cfg.finalizer.max_closure_overdue;
            let mut finalize_count = self.finalize_in_flight.len();

            for ch in &all_channels {
                if finalize_count >= self.cfg.finalizer.finalize_max_concurrent {
                    break;
                }
                if self.finalize_in_flight.contains(ch.get_id()) {
                    continue;
                }
                if let ChannelStatus::PendingToClose(closure_time) = ch.status {
                    let elapsed = closure_time.elapsed().unwrap_or(Duration::ZERO);
                    if elapsed >= overdue && self.try_finalize_channel(ch) {
                        finalize_count += 1;
                    }
                }
            }
        }

        // ── 5. Open pass ──────────────────────────────────────────────────────
        let deficit = self.cfg.population.target_open_channels.saturating_sub(open_count);

        if deficit == 0 {
            return Ok(());
        }

        if self.cfg.funding.stop_when_unfunded && safe_balance < self.cfg.funding.initial_balance {
            debug!(%safe_balance, "channel-lifecycle: safe balance too low to open new channels");
            return Ok(());
        }

        let existing_dests: HashSet<Address> = all_channels.iter().map(|c| c.destination).collect();
        let connected = self.node.network_view().connected_peers();

        let mut candidates: Vec<(Address, OffchainPublicKey, f64)> = connected
            .into_iter()
            .filter_map(|peer_id| {
                let &(offchain_key, chain_addr) = peer_addr_map.get(&peer_id)?;
                if chain_addr == me {
                    return None;
                }
                if existing_dests.contains(&chain_addr) {
                    return None;
                }
                if self.open_in_flight.contains(&chain_addr) {
                    return None;
                }
                if self
                    .cooldown
                    .get(&chain_addr)
                    .is_some_and(|until| Instant::now() < *until)
                {
                    return None;
                }
                if self
                    .cfg
                    .eligibility
                    .allowlist
                    .as_ref()
                    .is_some_and(|l| !l.contains(&chain_addr))
                {
                    return None;
                }
                if self.cfg.eligibility.blocklist.contains(&chain_addr) {
                    return None;
                }
                if self.cfg.eligibility.require_currently_connected && !self.node.network_view().is_connected(&peer_id)
                {
                    return None;
                }

                let score = self.peer_score_for(&offchain_key, &chain_addr);
                if score < self.cfg.eligibility.min_peer_quality_score {
                    return None;
                }

                Some((chain_addr, offchain_key, score))
            })
            .collect();

        candidates.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        candidates.truncate(deficit);

        let mut safe_remaining = safe_balance;
        for (addr, _, _) in candidates {
            if safe_remaining < self.cfg.funding.initial_balance {
                break;
            }
            if self.try_open_channel(addr, self.cfg.funding.initial_balance) {
                safe_remaining -= self.cfg.funding.initial_balance;
            }
        }

        Ok(())
    }

    /// Returns `true` if the channel meets the criteria for closure.
    ///
    /// `addr_to_key` maps chain addresses to offchain public keys, built from
    /// the account stream in `pipeline_inner`.
    fn should_close(&self, ch: &ChannelEntry, addr_to_key: &HashMap<Address, OffchainPublicKey>) -> bool {
        if ch.balance <= self.cfg.closure.close_when_drained_below {
            return true;
        }

        let dest = ch.destination;
        if let Some(pk) = addr_to_key.get(&dest) {
            let score = self.peer_score_for(pk, &dest);
            if score < self.cfg.closure.close_below_quality_score {
                return true;
            }

            let my_key = self.node.graph().identity();
            if let Some(edge) = self.node.graph().edge(my_key, pk) {
                let stale = edge.last_update() > self.cfg.closure.close_when_peer_unseen_for;
                let guard_passed = !self.cfg.eligibility.require_observed_since_start
                    || self.start_epoch.elapsed() >= self.cfg.restart.startup_close_grace_period;
                if stale && guard_passed {
                    return true;
                }
            }
        }

        false
    }
}
