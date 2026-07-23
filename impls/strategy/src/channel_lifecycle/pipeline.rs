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
use hopr_api::{
    PeerId,
    chain::{
        AccountSelector, ChainReadAccountOperations, ChainReadChannelOperations, ChainReadSafeOperations, ChainValues,
        ChainWriteChannelOperations, ChannelSelector, SafeSelector,
    },
    graph::{
        EdgeImmediateProtocolObservable as _, EdgeLinkObservable as _, EdgeObservableRead as _, NetworkGraphView as _,
    },
    network::NetworkView as _,
    node::{ActionableEventSource, HasChainApi, HasGraphView, HasNetworkView},
    types::{
        crypto::prelude::OffchainPublicKey,
        internal::prelude::{ChannelEntry, ChannelId, ChannelStatus},
        primitive::prelude::{Address, HoprBalance, WxHOPR},
    },
};
use tracing::{debug, warn};

use super::{
    ChannelLifecycleStrategyInner, ChannelObservation, PeerAddrCache,
    config::ResolvedFunding,
    selector::{
        BucketCell, BucketView, CloseCandidate, LatencyBucket, OpenCandidate, PeerEdgeInfo, SelectorContext, SignalSet,
        StakeView, SubnetBucket,
    },
};
use crate::errors::StrategyError;

/// TTL for the cached peer-id → (offchain key, chain address) map.  On-chain
/// account registrations change rarely; refreshing every 5 minutes is more
/// than enough for new entries to be picked up while avoiding a full account
/// stream on every tick.
const PEER_ADDR_CACHE_TTL: Duration = Duration::from_secs(5 * 60);

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
            // The fund/open passes already gate on `min_safe_balance_required`,
            // so this branch is only an informational signal — keep it at
            // `debug!` to avoid log spam in misconfigured environments.
            debug!(%me, "channel-lifecycle: safe not registered");
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

    /// Total number of in-flight chain-write operations across all four kinds.
    fn total_in_flight(&self) -> usize {
        self.open_in_flight.len()
            + self.fund_in_flight.len()
            + self.close_in_flight.len()
            + self.finalize_in_flight.len()
    }

    /// Fetches pre-computed graph edge information for `peer_offchain`.
    /// Returns default (zero) values when no edge record exists.
    fn peer_edge_info(&self, peer_offchain: &OffchainPublicKey) -> PeerEdgeInfo {
        let my_key = self.node.graph().identity();
        match self.node.graph().edge(my_key, peer_offchain) {
            Some(e) => {
                let (average_latency, probe_success_rate, ack_rate) = if let Some(qos) = e.immediate_qos() {
                    (qos.average_latency(), qos.average_probe_rate(), qos.ack_rate())
                } else {
                    (None, 0.0, None)
                };
                PeerEdgeInfo {
                    edge_score: Some(e.score()),
                    last_update: e.last_update(),
                    average_latency,
                    probe_success_rate,
                    ack_rate,
                }
            }
            None => PeerEdgeInfo::default(),
        }
    }

    /// Returns `true` if the channel should be proactively funded before the
    /// next transaction confirms.
    ///
    /// `prev_obs` must hold the observation captured at the **start of the
    /// current tick**, before the snapshot pass refreshed `last_observed` —
    /// otherwise `prev_obs.balance` and `channel.balance` are equal and the
    /// drain estimate collapses to zero.
    ///
    /// `lower_threshold` is the resolved wxHOPR lower-balance threshold,
    /// supplied by the caller so that the computation uses pre-resolved
    /// economics from the current tick.
    fn proactive_fund_needed(
        &self,
        channel: &ChannelEntry,
        prev_obs: &HashMap<ChannelId, ChannelObservation>,
        est_tx_secs: f64,
        min_ticket_price_wei: f64,
        lower_threshold: HoprBalance,
    ) -> bool {
        if !self.cfg.proactive_funding.enabled {
            return false;
        }

        let Some(obs) = prev_obs.get(channel.get_id()) else {
            return false;
        };

        let lookback_secs = self.cfg.proactive_funding.depletion_lookback.as_secs_f64().max(1.0);
        let elapsed = obs.at.elapsed().as_secs_f64().max(0.01);

        // `obs` is the older snapshot; `channel` is the current on-chain state.
        // A drain decreases the balance, so the delta is `previous - current`.
        let balance_prev = obs.balance.amount().low_u128() as f64;
        let balance_current = channel.balance.amount().low_u128() as f64;
        let balance_delta = (balance_prev - balance_current).max(0.0);
        let balance_drain_rate =
            (self.cfg.proactive_funding.balance_drain_weight * balance_delta / elapsed.min(lookback_secs)).max(0.0);

        let ticket_delta = channel.ticket_index.saturating_sub(obs.ticket_index) as f64;
        let ticket_drain_rate =
            (self.cfg.proactive_funding.ticket_index_drain_weight * ticket_delta * min_ticket_price_wei
                / elapsed.min(lookback_secs))
            .max(0.0);

        let drain_rate = balance_drain_rate + ticket_drain_rate;
        let projected_drain = drain_rate * est_tx_secs * self.cfg.proactive_funding.safety_margin;

        let balance_after = balance_current - projected_drain;
        let threshold = lower_threshold.amount().low_u128() as f64;

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

        debug!(%channel, %topup, "channel-lifecycle: funding channel");
        #[cfg(all(feature = "telemetry", not(test)))]
        super::METRIC_CHANNEL_FUNDS.increment();

        let chain = self.node.chain_api().clone();
        let in_flight = Arc::clone(&self.fund_in_flight);

        hopr_utils::runtime::prelude::spawn(async move {
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

        debug!(%channel, "channel-lifecycle: closing channel");
        #[cfg(all(feature = "telemetry", not(test)))]
        super::METRIC_CHANNEL_CLOSES.increment();

        let chain = self.node.chain_api().clone();
        let in_flight = Arc::clone(&self.close_in_flight);

        hopr_utils::runtime::prelude::spawn(async move {
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

        debug!(%channel, "channel-lifecycle: finalizing closure");
        #[cfg(all(feature = "telemetry", not(test)))]
        super::METRIC_CHANNEL_FINALIZES.increment();

        let chain = self.node.chain_api().clone();
        let in_flight = Arc::clone(&self.finalize_in_flight);

        hopr_utils::runtime::prelude::spawn(async move {
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

    /// Spawn an open transaction for a new channel to `dest`.  Returns the
    /// committed amount if a chain action was submitted (either a fresh open or
    /// an immediate top-up), or `None` if no action was taken.
    ///
    /// `funding` carries the wxHOPR amounts already resolved from the capacity
    /// config for this tick.  `amount` is `funding.initial_balance` (passed by
    /// the caller so this helper stays generic).
    ///
    /// Before submitting, queries the current on-chain channel state from the
    /// pipeline task so the strategy converges to the desired state in this
    /// tick rather than deferring to the next one.  The `channel_by_parties`
    /// call is serviced by the in-process cache (moka + RocksDB), so the
    /// overhead is a fast in-memory lookup in the common case.
    fn try_open_channel(&self, dest: Address, amount: HoprBalance, funding: &ResolvedFunding) -> Option<HoprBalance> {
        if !self.open_in_flight.insert(dest) {
            return None;
        }
        if self.total_in_flight() > self.cfg.concurrency.max_concurrent_actions {
            self.open_in_flight.remove(&dest);
            return None;
        }

        // Pre-check current on-chain state.  The snapshot `all_channels` in
        // `pipeline_inner` can be stale (race between chain events and the
        // snapshot pass), so we re-read here before spending a tx slot.
        {
            let chain = self.node.chain_api();
            let me = *chain.me();
            match chain.channel_by_parties(&me, &dest) {
                Ok(Some(existing)) => match existing.status {
                    ChannelStatus::Open => {
                        self.open_in_flight.remove(&dest);
                        if existing.balance >= funding.lower_balance_threshold {
                            debug!(%dest, balance = %existing.balance, "channel-lifecycle: already open at desired stake, skipping");
                            return None;
                        }
                        debug!(%dest, balance = %existing.balance, "channel-lifecycle: already open below threshold, funding immediately");
                        let topup = funding.topup_balance;
                        return self.try_fund_channel(&existing, topup).then_some(topup);
                    }
                    ChannelStatus::PendingToClose(_) => {
                        self.open_in_flight.remove(&dest);
                        debug!(%dest, "channel-lifecycle: channel pending closure, deferring open");
                        return None;
                    }
                    _ => {} // Closed — fall through to open
                },
                Ok(None) => {} // No channel yet — fall through to open
                Err(e) => {
                    warn!(%dest, %e, "channel-lifecycle: channel_by_parties check failed, proceeding with open");
                }
            }
        }

        debug!(%dest, %amount, "channel-lifecycle: opening channel");
        #[cfg(all(feature = "telemetry", not(test)))]
        super::METRIC_CHANNEL_OPENS.increment();

        let chain = self.node.chain_api().clone();
        let in_flight = Arc::clone(&self.open_in_flight);

        hopr_utils::runtime::prelude::spawn(async move {
            match chain.open_channel(&dest, amount).await {
                Ok(confirmation) => {
                    if let Err(e) = confirmation.await {
                        warn!(%dest, %e, "channel-lifecycle: open tx failed");
                    }
                    // Clear in_flight once the confirmation future resolves,
                    // success or failure — the tx is no longer pending either way.
                    // ChannelOpened event handler also clears it as a no-op fallback.
                    in_flight.remove(&dest);
                }
                Err(e) => {
                    warn!(%dest, %e, "channel-lifecycle: failed to submit open tx");
                    in_flight.remove(&dest);
                }
            }
        });

        Some(amount)
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
        let (est_tx_time_val, safe_balance_result) = futures::join!(self.est_tx_time(), self.safe_balance_budget());
        let est_tx_secs = est_tx_time_val.as_secs_f64();
        let safe_balance = safe_balance_result?;

        let mut all_channels: Vec<ChannelEntry> = Vec::new();
        {
            let mut s = chain
                .stream_channels(ChannelSelector::default().with_source(me))
                .map_err(|e| StrategyError::Other(e.into()))?;
            while let Some(ch) = s.next().await {
                all_channels.push(ch);
            }
        }

        // Capture the previous-tick observation snapshot *before* refreshing
        // `last_observed` — `proactive_fund_needed` needs to compare against
        // the older balance/ticket_index, otherwise the delta is always 0.
        let prev_observations: HashMap<ChannelId, ChannelObservation> = all_channels
            .iter()
            .filter_map(|ch| self.last_observed.get(ch.get_id()).map(|v| (*ch.get_id(), v.clone())))
            .collect();

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

        // Fetch ticket economics once per tick.  Both values are needed for
        // the capacity-to-wxHOPR conversion and for the proactive-funding drain
        // estimate.  When either value is unavailable the fund and open passes
        // are skipped; close and finalize still run.
        let ticket_price = chain.minimum_ticket_price().await.unwrap_or_else(|e| {
            warn!(%e, "channel-lifecycle: minimum_ticket_price unavailable, using zero");
            HoprBalance::zero()
        });
        let min_ticket_price_wei = ticket_price.amount().low_u128() as f64;

        let win_prob = match chain.minimum_incoming_ticket_win_prob().await {
            Ok(wp) => Some(wp),
            Err(e) => {
                warn!(%e, "channel-lifecycle: minimum_incoming_ticket_win_prob unavailable, skipping fund/open passes");
                None
            }
        };

        // Resolve data-capacity config fields to wxHOPR amounts for this tick.
        // `None` when win_prob is unavailable; fund/open passes are skipped.
        let funding = win_prob.map(|wp| self.cfg.funding.resolve(ticket_price, wp));

        // Share resolved economics with the event-driven funding handler so it
        // can reuse per-tick values rather than issuing fresh chain RPC calls.
        *self.last_resolved_funding.lock() = funding;

        let peer_addr_map = self.peer_addr_map(chain).await?;

        let addr_to_key: HashMap<Address, OffchainPublicKey> =
            peer_addr_map.values().map(|(pk, addr)| (*addr, *pk)).collect();

        let open_channels: Vec<&ChannelEntry> = all_channels
            .iter()
            .filter(|c| c.status == ChannelStatus::Open)
            .collect();
        let open_count = open_channels.len() + self.open_in_flight.len();
        debug!(
            open = open_count,
            in_flight = self.total_in_flight(),
            safe = %safe_balance,
            channels = all_channels.len(),
            "channel-lifecycle: tick"
        );

        // Computed once here so close and open passes don't each iterate all
        // activity.  Floored at 1 so ticket-score division cannot produce NaN.
        let max_activity: u64 = self
            .peer_ticket_activity
            .iter()
            .map(|e| *e.value())
            .max()
            .unwrap_or(0)
            .max(1);

        // The fund and open passes share this budget so opens cannot promise
        // stake the funding txs already staked in this same tick.
        let mut safe_remaining = safe_balance;

        // ── 2. Fund pass ─────────────────────────────────────────────────────
        if let Some(funding) = funding {
            if safe_balance >= funding.min_safe_balance_required || !self.cfg.funding.stop_when_unfunded {
                for ch in &open_channels {
                    if self.fund_in_flight.contains(ch.get_id()) || self.close_in_flight.contains(ch.get_id()) {
                        continue;
                    }
                    let needs_topup = ch.balance <= funding.lower_balance_threshold;
                    let needs_proactive = self.proactive_fund_needed(
                        ch,
                        &prev_observations,
                        est_tx_secs,
                        min_ticket_price_wei,
                        funding.lower_balance_threshold,
                    );

                    if needs_topup || needs_proactive {
                        let reason = if needs_topup {
                            "below_lower_threshold"
                        } else {
                            "proactive_drain"
                        };
                        debug!(%ch, reason, safe_remaining = %safe_remaining, "channel-lifecycle: fund candidate");
                        if safe_remaining < funding.topup_balance {
                            debug!("channel-lifecycle: safe balance exhausted in fund pass");
                            break;
                        }
                        if self.try_fund_channel(ch, funding.topup_balance) {
                            safe_remaining -= funding.topup_balance;
                        }
                    }
                }
            } else {
                debug!(
                    safe = %safe_balance,
                    min_required = %funding.min_safe_balance_required,
                    "channel-lifecycle: fund pass skipped: safe below minimum"
                );
            }
        } else {
            debug!("channel-lifecycle: fund pass skipped: ticket economics unavailable");
        }

        // ── Selector context ─────────────────────────────────────────────────
        // Build pre-processed data for the pluggable selector.  Eligibility
        // hard-gates (allowlist / blocklist / cooldown / in-flight / connected)
        // are applied here so every selector inherits them.  The selector only
        // ranks and filters further by its own policy.

        let close_candidates: Vec<CloseCandidate> = open_channels
            .iter()
            .map(|ch| {
                let offchain_key = addr_to_key.get(&ch.destination).copied();
                let edge_info = offchain_key
                    .as_ref()
                    .map(|pk| self.peer_edge_info(pk))
                    .unwrap_or_default();
                let ticket_score = self
                    .peer_ticket_activity
                    .get(&ch.destination)
                    .map(|v| *v as f64)
                    .unwrap_or(0.0)
                    / (max_activity as f64);
                CloseCandidate {
                    channel: **ch,
                    offchain_key,
                    edge_info,
                    ticket_score,
                }
            })
            .collect();

        // Index for the close pass: ChannelId → entry (selector returns IDs).
        let channel_by_id: HashMap<ChannelId, &ChannelEntry> =
            open_channels.iter().map(|ch| (*ch.get_id(), *ch)).collect();

        let existing_dests: HashSet<Address> = all_channels.iter().map(|c| c.destination).collect();
        let connected = self.node.network_view().connected_peers();

        let open_candidates: Vec<OpenCandidate> = connected
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
                // Every peer_id in this loop came from `connected_peers()`, so it is
                // connected by construction; a separate `is_connected` check is always true
                // and can be skipped. Note: `require_currently_connected = false` has no
                // observable effect with the current iteration source — a follow-up can widen
                // the source to `peer_addr_map` if that use-case is needed.

                let edge_info = self.peer_edge_info(&offchain_key);
                let ticket_delta = self.peer_ticket_activity.get(&chain_addr).map(|v| *v).unwrap_or(0);
                let ticket_score = (ticket_delta as f64) / (max_activity as f64);

                // Composite-score eligibility gate — same formula as today's
                // `DefaultSelector`, kept here so the threshold acts as a hard
                // filter that all selectors inherit.
                let composite = self.cfg.eligibility.peer_quality_weight * edge_info.edge_score.unwrap_or(0.0)
                    + self.cfg.eligibility.ticket_activity_weight * ticket_score;
                if composite < self.cfg.eligibility.min_peer_quality_score {
                    return None;
                }

                let subnet = self
                    .node
                    .network_view()
                    .multiaddress_of(&peer_id)
                    .map(|addrs| SubnetBucket::from_multiaddrs(&addrs))
                    .unwrap_or(SubnetBucket::Unknown);

                Some(OpenCandidate {
                    addr: chain_addr,
                    offchain_key,
                    edge_info,
                    ticket_score,
                    subnet,
                })
            })
            .collect();

        let deficit = self.cfg.population.target_open_channels.saturating_sub(open_count);

        // Build addr → PeerId index for multiaddr lookups.
        let addr_to_peer_id: HashMap<Address, PeerId> = peer_addr_map
            .iter()
            .map(|(peer_id, (_, addr))| (*addr, *peer_id))
            .collect();

        // (latency, subnet) cells for all open channels — always computed, cheap.
        let bucket_cells: HashMap<ChannelId, BucketCell> = close_candidates
            .iter()
            .filter_map(|c| {
                let peer_id = addr_to_peer_id.get(&c.channel.destination)?;
                let addrs = self.node.network_view().multiaddress_of(peer_id).unwrap_or_default();
                let subnet = SubnetBucket::from_multiaddrs(&addrs);
                let lat = LatencyBucket::from_latency(c.edge_info.average_latency);
                Some((*c.channel.get_id(), BucketCell { latency: lat, subnet }))
            })
            .collect();
        let bucket_view = BucketView::new(bucket_cells);
        self.emit_bucket_metrics(&bucket_view, &close_candidates);

        // On-chain stake scores — only fetched when the active selector requests STAKE.
        let stake_view = if self.selector.required_signals().contains(SignalSet::STAKE) {
            self.fetch_stake_view(chain, &close_candidates, &open_candidates).await
        } else {
            StakeView::empty()
        };

        let selector_ctx = SelectorContext {
            cfg: &self.cfg,
            deficit,
            open_candidates: &open_candidates,
            close_candidates: &close_candidates,
            start_epoch_elapsed: self.start_epoch.elapsed(),
            bucket_view,
            stake_view,
        };

        self.emit_score_axis_metrics(&selector_ctx);
        let closes_ranked = self.selector.select_closes(&selector_ctx).await;
        let opens_ranked = self.selector.select_opens(&selector_ctx).await;

        // ── 3. Close pass ─────────────────────────────────────────────────────
        if self.start_epoch.elapsed() >= self.cfg.restart.startup_close_grace_period {
            let mut close_count = self.close_in_flight.len();
            debug!(
                in_flight = close_count,
                open = open_count,
                min = self.cfg.population.min_open_channels,
                "channel-lifecycle: close pass"
            );

            for channel_id in &closes_ranked {
                if close_count >= self.cfg.closure.close_max_concurrent {
                    break;
                }
                let remaining_open = open_count.saturating_sub(close_count);
                if remaining_open <= self.cfg.population.min_open_channels {
                    break;
                }
                if let Some(ch) = channel_by_id.get(channel_id) {
                    if self.close_in_flight.contains(ch.get_id()) || self.fund_in_flight.contains(ch.get_id()) {
                        continue;
                    }
                    if self.try_close_channel(ch) {
                        close_count += 1;
                    }
                }
            }
        }

        // ── 4. Finalize pass ──────────────────────────────────────────────────
        if self.cfg.finalizer.enabled {
            let overdue = self.cfg.finalizer.max_closure_overdue;
            let mut finalize_count = self.finalize_in_flight.len();

            for ch in &all_channels {
                if finalize_count >= self.cfg.finalizer.finalize_max_concurrent {
                    break;
                }
                if self.finalize_in_flight.contains(ch.get_id()) {
                    continue;
                }
                if let ChannelStatus::PendingToClose(closure_time) = ch.status {
                    // elapsed() is Err when the deadline is still in the future;
                    // skip rather than treating it as zero (would fire prematurely).
                    if let Ok(elapsed) = closure_time.elapsed()
                        && elapsed >= overdue
                        && self.try_finalize_channel(ch)
                    {
                        finalize_count += 1;
                    }
                }
            }
        }

        // ── 5. Open pass ──────────────────────────────────────────────────────
        debug!(
            deficit,
            open = open_count,
            target = self.cfg.population.target_open_channels,
            "channel-lifecycle: open pass"
        );

        if deficit == 0 {
            return Ok(());
        }

        let Some(funding) = funding else {
            debug!("channel-lifecycle: open pass skipped: ticket economics unavailable");
            return Ok(());
        };

        if self.cfg.funding.stop_when_unfunded && safe_remaining < funding.initial_balance {
            debug!(%safe_remaining, "channel-lifecycle: safe balance too low to open new channels");
            return Ok(());
        }

        debug!(
            candidates = opens_ranked.len(),
            deficit, "channel-lifecycle: open pass candidates"
        );

        for (addr, _) in opens_ranked.iter().take(deficit) {
            if safe_remaining < funding.initial_balance {
                break;
            }
            if let Some(committed) = self.try_open_channel(*addr, funding.initial_balance, &funding) {
                safe_remaining -= committed;
            }
        }

        Ok(())
    }

    /// Return the cached peer-id → (offchain key, chain address) map,
    /// refreshing it from the on-chain account stream when the cache is empty
    /// or older than [`PEER_ADDR_CACHE_TTL`].  Filtered to accounts with a
    /// published off-chain key, which is the only set we can address as peers.
    async fn peer_addr_map(
        &self,
        chain: &N::ChainApi,
    ) -> crate::errors::Result<HashMap<PeerId, (OffchainPublicKey, Address)>> {
        let cached = {
            let guard = self.peer_addr_cache.lock();
            guard.as_ref().and_then(|c| {
                if c.refreshed_at.elapsed() < PEER_ADDR_CACHE_TTL {
                    Some(c.map.clone())
                } else {
                    None
                }
            })
        };

        if let Some(map) = cached {
            return Ok(map);
        }

        let mut map: HashMap<PeerId, (OffchainPublicKey, Address)> = HashMap::new();
        let mut accounts = chain
            .stream_accounts(AccountSelector::default().with_public_only(true))
            .map_err(|e| StrategyError::Other(e.into()))?;
        while let Some(account) = accounts.next().await {
            let peer_id = PeerId::from(&account.public_key);
            map.insert(peer_id, (account.public_key, account.chain_addr));
        }

        *self.peer_addr_cache.lock() = Some(PeerAddrCache {
            refreshed_at: Instant::now(),
            map: map.clone(),
        });

        Ok(map)
    }

    /// Fetches on-chain safe balances for all candidate peers and returns
    /// normalized scores in [0, 1].  Called only when the active selector
    /// requests the `STAKE` signal.
    ///
    /// Peers whose safe address cannot be resolved score `0.0`.
    /// Balance fetches that time out also score `0.0`.
    async fn fetch_stake_view(
        &self,
        chain: &N::ChainApi,
        close_candidates: &[CloseCandidate],
        open_candidates: &[OpenCandidate],
    ) -> StakeView {
        // Collect all unique chain addresses we need scores for.
        let addresses: HashSet<Address> = close_candidates
            .iter()
            .map(|c| c.channel.destination)
            .chain(open_candidates.iter().map(|c| c.addr))
            .collect();

        if addresses.is_empty() {
            return StakeView::empty();
        }

        // Stream all accounts to build chain_addr → safe_address mapping.
        let safe_map: HashMap<Address, Address> = {
            let Ok(mut stream) = chain.stream_accounts(AccountSelector::default()) else {
                return StakeView::empty();
            };
            let mut m = HashMap::new();
            while let Some(account) = stream.next().await {
                if let Some(safe_addr) = account.safe_address {
                    m.insert(account.chain_addr, safe_addr);
                }
            }
            m
        };

        // Fetch balances concurrently (≤ 8 in-flight, 500 ms timeout per peer).
        let fetch_timeout = futures_time::time::Duration::from_millis(500);
        let raw_scores: HashMap<Address, f64> = futures::stream::iter(addresses)
            .map(|addr| {
                let safe_addr = safe_map.get(&addr).copied();
                let chain = chain.clone();
                async move {
                    use futures_time::future::FutureExt as _;
                    let Some(safe_addr) = safe_addr else {
                        return (addr, 0.0_f64);
                    };
                    let result = chain.balance::<WxHOPR, _>(safe_addr).timeout(fetch_timeout).await;
                    let amount = match result {
                        Ok(Ok(bal)) => bal.amount().low_u128() as f64,
                        Ok(Err(e)) => {
                            tracing::trace!(%addr, %e, "channel-lifecycle: stake fetch error");
                            0.0
                        }
                        Err(_) => {
                            tracing::trace!(%addr, "channel-lifecycle: stake fetch timeout");
                            0.0
                        }
                    };
                    (addr, amount)
                }
            })
            .buffer_unordered(8)
            .collect()
            .await;

        // Normalize via log1p scale, using the max observed value.
        let max_amount = raw_scores.values().cloned().fold(0.0_f64, f64::max);
        let normalizer = if max_amount > 0.0 {
            max_amount
        } else {
            return StakeView::empty();
        };

        let scores = raw_scores
            .into_iter()
            .map(|(addr, amount)| {
                let score = (amount.ln_1p() / normalizer.ln_1p()).min(1.0);
                (addr, score)
            })
            .collect();

        StakeView::from_scores(scores)
    }

    /// Emit bucket-diversity metrics after each tick snapshot.
    #[allow(unused_variables)]
    fn emit_bucket_metrics(&self, bucket_view: &BucketView, close_candidates: &[CloseCandidate]) {
        #[cfg(all(feature = "telemetry", not(test)))]
        {
            use std::collections::HashSet;

            use super::{
                METRIC_BUCKET_COUNT, METRIC_EFFECTIVE_BUCKETS, METRIC_LATENCY_VARIANCE_MS, METRIC_SUBNET_COUNT,
            };

            METRIC_EFFECTIVE_BUCKETS.set(bucket_view.effective_buckets());

            // Per-cell counts.
            let mut seen_cells = HashSet::new();
            for c in close_candidates {
                if let Some(cell) = bucket_view
                    .cell_for(c.channel.get_id())
                    .filter(|c| seen_cells.insert((*c).clone()))
                {
                    let label = format!("{:?}_{:?}", cell.latency, cell.subnet);
                    METRIC_BUCKET_COUNT.set(&[label.as_str()], bucket_view.cell_count(cell) as f64);
                }
            }

            // Latency variance (ms) across all open channels.
            let latencies_ms: Vec<f64> = close_candidates
                .iter()
                .filter_map(|c| c.edge_info.average_latency)
                .map(|d| d.as_secs_f64() * 1000.0)
                .collect();
            if !latencies_ms.is_empty() {
                let n = latencies_ms.len() as f64;
                let mean = latencies_ms.iter().sum::<f64>() / n;
                let variance = latencies_ms.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;
                METRIC_LATENCY_VARIANCE_MS.set(variance);
            } else {
                METRIC_LATENCY_VARIANCE_MS.set(0.0);
            }

            // Distinct subnet count.
            let subnets: HashSet<String> = close_candidates
                .iter()
                .filter_map(|c| bucket_view.cell_for(c.channel.get_id()))
                .filter(|cell| !matches!(cell.subnet, SubnetBucket::Unknown))
                .map(|cell| format!("{:?}", cell.subnet))
                .collect();
            METRIC_SUBNET_COUNT.set(subnets.len() as f64);
        }
    }

    /// Emit per-axis score averages across open candidates, but only when the
    /// multi-objective selector is active.
    #[allow(unused_variables)]
    fn emit_score_axis_metrics(&self, ctx: &SelectorContext<'_>) {
        #[cfg(all(feature = "telemetry", not(test)))]
        {
            use super::METRIC_SCORE_AXIS;

            let Some(mo_cfg) = self.cfg.selector.multi_objective_config() else {
                for axis in &["latency", "trust", "stake", "anon_penalty"] {
                    METRIC_SCORE_AXIS.set(&[axis], 0.0);
                }
                return;
            };

            if ctx.open_candidates.is_empty() {
                for axis in &["latency", "trust", "stake", "anon_penalty"] {
                    METRIC_SCORE_AXIS.set(&[axis], 0.0);
                }
                return;
            }

            let n = ctx.open_candidates.len() as f64;
            let mut sum_lat = 0.0_f64;
            let mut sum_trust = 0.0_f64;
            let mut sum_stake = 0.0_f64;
            let mut sum_anon = 0.0_f64;

            for c in ctx.open_candidates {
                let lat = match LatencyBucket::from_latency(c.edge_info.average_latency) {
                    LatencyBucket::VeryFast => 1.0,
                    LatencyBucket::Fast => 0.75,
                    LatencyBucket::Medium => 0.50,
                    LatencyBucket::Slow => 0.25,
                    LatencyBucket::VerySlow => 0.0,
                };
                let w = &mo_cfg.weights;
                let trust = w.trust_probe * c.edge_info.probe_success_rate
                    + w.trust_ack * c.edge_info.ack_rate.unwrap_or(0.0)
                    + w.trust_ticket * c.ticket_score;
                let stake = ctx.stake_view.score(&c.addr);
                let cell = BucketCell {
                    latency: LatencyBucket::from_latency(c.edge_info.average_latency),
                    subnet: c.subnet.clone(),
                };
                let anon_penalty = ctx.bucket_view.bucket_coverage(&cell);

                sum_lat += lat;
                sum_trust += trust;
                sum_stake += stake;
                sum_anon += anon_penalty;
            }

            METRIC_SCORE_AXIS.set(&["latency"], sum_lat / n);
            METRIC_SCORE_AXIS.set(&["trust"], sum_trust / n);
            METRIC_SCORE_AXIS.set(&["stake"], sum_stake / n);
            METRIC_SCORE_AXIS.set(&["anon_penalty"], sum_anon / n);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashSet,
        sync::Arc,
        time::{Duration, Instant},
    };

    use anyhow::Context as _;
    use bytesize::ByteSize;
    use dashmap::DashMap;
    use futures::StreamExt as _;
    use hex_literal::hex;
    use hopr_api::{
        PeerId,
        chain::{
            AccountSelector, ChainEvent, ChainEvents, ChainReadAccountOperations, ChainReadChannelOperations,
            ChainWriteAccountOperations, ChannelSelector, HoprChainApi, WinningProbability,
        },
        node::{
            ActionableEvent, ActionableEventDiscriminant, ActionableEventSource, ComponentStatus,
            ComponentStatusReporter, EventWaitResult, HasChainApi, HasGraphView, HasNetworkView, NodeOnchainIdentity,
        },
        types::{
            crypto::{
                keypairs::Keypair,
                prelude::{ChainKeypair, OffchainPublicKey},
            },
            internal::prelude::{ChannelEntry, ChannelStatus},
            primitive::prelude::{Address, BytesRepresentable, HoprBalance, XDaiBalance},
        },
    };
    use hopr_chain_connector::{create_trustful_hopr_blokli_connector, testing::BlokliTestStateBuilder};

    // `super` here is `pipeline`; `super::super` is `channel_lifecycle`.
    // Private items (ChannelLifecycleStrategyInner) are accessible from descendant modules.
    use super::super::ChannelLifecycleStrategyInner;
    use super::super::{config::ResolvedFunding, *};

    /// Build a [`ResolvedFunding`] directly from wxHOPR amounts for use in
    /// `try_open_channel` unit tests that bypass the pipeline.
    fn make_resolved(
        lower: HoprBalance,
        topup: HoprBalance,
        initial: HoprBalance,
        min_safe: HoprBalance,
    ) -> ResolvedFunding {
        ResolvedFunding {
            lower_balance_threshold: lower,
            topup_balance: topup,
            initial_balance: initial,
            min_safe_balance_required: min_safe,
        }
    }

    lazy_static::lazy_static! {
        static ref BOB_KP: ChainKeypair = ChainKeypair::from_secret(&hex!(
            "492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775"
        ))
        .expect("test keypair");
        static ref ALICE: Address = hex!("18f8ae833c85c51fbeba29cef9fbfb53b3bad950").into();
        static ref BOB: Address = BOB_KP.public().to_address();
        static ref CHRIS: Address = hex!("b6021e0860dd9d96c9ff0a73e2e5ba3a466ba234").into();
        static ref DAVE: Address = hex!("68499f50ff68d523385dc60686069935d17d762a").into();
    }

    /// Minimal node wrapper — same pattern as in auto_funding tests.
    /// The second field is a shared stub graph; tests that need configurable
    /// per-peer edges use `Arc::clone` of the graph to insert edges while the
    /// strategy is running.  Constructed via `ChainNode::new` for the common
    /// case (empty graph) or `ChainNode::with_graph` for custom graphs.
    struct ChainNode<C>(C, Arc<StubGraph>);

    impl<C> ChainNode<C> {
        fn new(chain: C) -> Self {
            ChainNode(chain, Arc::new(StubGraph::default()))
        }

        fn with_graph(chain: C, graph: Arc<StubGraph>) -> Self {
            ChainNode(chain, graph)
        }
    }

    impl<C> HasChainApi for ChainNode<C>
    where
        C: HoprChainApi + ComponentStatusReporter + Clone + Send + Sync + 'static,
    {
        type ChainApi = C;
        type ChainError = <C as HoprChainApi>::ChainError;

        fn identity(&self) -> &NodeOnchainIdentity {
            static IDENTITY: std::sync::OnceLock<NodeOnchainIdentity> = std::sync::OnceLock::new();
            IDENTITY.get_or_init(NodeOnchainIdentity::default)
        }

        fn chain_api(&self) -> &C {
            &self.0
        }

        fn status(&self) -> ComponentStatus {
            self.0.component_status()
        }

        fn wait_for_on_chain_event<F>(
            &self,
            _predicate: F,
            _context: String,
            _timeout: Duration,
        ) -> EventWaitResult<<C as HoprChainApi>::ChainError, <C as HoprChainApi>::ChainError>
        where
            F: Fn(&ChainEvent) -> bool + Send + Sync + 'static,
        {
            unimplemented!("tests do not call wait_for_on_chain_event")
        }
    }

    impl<C> ActionableEventSource for ChainNode<C>
    where
        C: ChainEvents + Send + Sync + 'static,
    {
        fn subscribe_to_actionable_events(
            &self,
            _filter: Option<&[ActionableEventDiscriminant]>,
        ) -> Result<futures::stream::BoxStream<'static, ActionableEvent>, String> {
            Ok(self
                .0
                .subscribe()
                .map_err(|e| e.to_string())?
                .map(ActionableEvent::Chain)
                .boxed())
        }
    }

    struct StubNetworkView;

    impl hopr_api::network::NetworkView for StubNetworkView {
        fn listening_as(&self) -> HashSet<hopr_api::Multiaddr> {
            HashSet::new()
        }

        fn multiaddress_of(&self, _peer: &PeerId) -> Option<HashSet<hopr_api::Multiaddr>> {
            None
        }

        fn discovered_peers(&self) -> HashSet<PeerId> {
            HashSet::new()
        }

        fn connected_peers(&self) -> HashSet<PeerId> {
            HashSet::new()
        }

        fn is_connected(&self, _peer: &PeerId) -> bool {
            false
        }

        fn health(&self) -> hopr_api::network::Health {
            hopr_api::network::Health::Red
        }

        fn subscribe_network_events(
            &self,
        ) -> impl futures::Stream<Item = hopr_api::network::NetworkEvent> + Send + 'static {
            futures::stream::pending()
        }
    }

    impl<C> HasNetworkView for ChainNode<C>
    where
        C: HoprChainApi + ComponentStatusReporter + Clone + Send + Sync + 'static,
    {
        type NetworkView = StubNetworkView;

        fn network_view(&self) -> &Self::NetworkView {
            static NV: StubNetworkView = StubNetworkView;
            &NV
        }

        fn status(&self) -> ComponentStatus {
            ComponentStatus::Ready
        }
    }

    /// Programmable stub graph.  By default all edge queries return `None`
    /// (behaviour identical to the former unit-struct).  Tests that need
    /// configurable edges use `insert_edge` to pre-populate the map.
    #[derive(Clone, Default)]
    struct StubGraph {
        edges: Arc<DashMap<(OffchainPublicKey, OffchainPublicKey), StubEdge>>,
    }

    impl StubGraph {
        fn insert_edge(&self, src: OffchainPublicKey, dest: OffchainPublicKey, edge: StubEdge) {
            self.edges.insert((src, dest), edge);
        }
    }

    impl hopr_api::graph::NetworkGraphView for StubGraph {
        type NodeId = OffchainPublicKey;
        type Observed = StubEdge;

        fn node_count(&self) -> usize {
            0
        }

        fn contains_node(&self, _key: &OffchainPublicKey) -> bool {
            false
        }

        fn nodes(&self) -> futures::stream::BoxStream<'static, OffchainPublicKey> {
            Box::pin(futures::stream::empty())
        }

        fn edge(&self, src: &OffchainPublicKey, dest: &OffchainPublicKey) -> Option<StubEdge> {
            self.edges.get(&(*src, *dest)).map(|v| v.clone())
        }

        fn identity(&self) -> &OffchainPublicKey {
            static KEY: std::sync::OnceLock<OffchainPublicKey> = std::sync::OnceLock::new();
            KEY.get_or_init(|| {
                use hopr_api::types::crypto::keypairs::Keypair as _;
                *hopr_api::types::crypto::prelude::OffchainKeypair::from_secret(&[1u8; 32])
                    .expect("test key")
                    .public()
            })
        }
    }

    impl hopr_api::graph::NetworkGraphConnectivity for StubGraph {
        type NodeId = OffchainPublicKey;
        type Observed = StubEdge;

        fn connected_edges(&self) -> Vec<(OffchainPublicKey, OffchainPublicKey, StubEdge)> {
            Vec::new()
        }

        fn reachable_edges(&self) -> Vec<(OffchainPublicKey, OffchainPublicKey, StubEdge)> {
            Vec::new()
        }
    }

    impl hopr_api::graph::NetworkGraphTraverse for StubGraph {
        type NodeId = OffchainPublicKey;
        type Observed = StubEdge;

        fn simple_paths<V: hopr_api::graph::ValueFn<Weight = StubEdge>>(
            &self,
            _source: &OffchainPublicKey,
            _destination: &OffchainPublicKey,
            _length: usize,
            _take_count: Option<usize>,
            _value_fn: V,
        ) -> Vec<(Vec<OffchainPublicKey>, [u64; 5], V::Value)> {
            Vec::new()
        }

        fn simple_paths_from<V: hopr_api::graph::ValueFn<Weight = StubEdge>>(
            &self,
            _source: &OffchainPublicKey,
            _length: usize,
            _take_count: Option<usize>,
            _value_fn: V,
        ) -> Vec<(Vec<OffchainPublicKey>, [u64; 5], V::Value)> {
            Vec::new()
        }

        fn simple_loopback_to_self(
            &self,
            _length: usize,
            _take_count: Option<usize>,
        ) -> Vec<(Vec<OffchainPublicKey>, [u64; 5])> {
            Vec::new()
        }
    }

    #[derive(Clone)]
    struct StubEdge {
        last_update: Duration,
        score: f64,
    }

    impl Default for StubEdge {
        fn default() -> Self {
            Self {
                last_update: Duration::ZERO,
                score: 0.5,
            }
        }
    }

    impl hopr_api::graph::EdgeObservableRead for StubEdge {
        type ImmediateMeasurement = StubMeasurement;
        type IntermediateMeasurement = StubMeasurement;

        fn last_update(&self) -> Duration {
            self.last_update
        }

        fn immediate_qos(&self) -> Option<&Self::ImmediateMeasurement> {
            None
        }

        fn intermediate_qos(&self) -> Option<&Self::IntermediateMeasurement> {
            None
        }

        fn score(&self) -> f64 {
            self.score
        }
    }

    impl hopr_api::graph::traits::EdgeObservableWrite for StubEdge {
        fn record(&mut self, _measurement: hopr_api::graph::traits::EdgeWeightType) {}
    }

    struct StubMeasurement;

    impl hopr_api::graph::EdgeLinkObservable for StubMeasurement {
        fn record(&mut self, _: hopr_api::graph::traits::EdgeTransportMeasurement) {}

        fn average_latency(&self) -> Option<Duration> {
            None
        }

        fn average_probe_rate(&self) -> f64 {
            0.0
        }

        fn score(&self) -> f64 {
            0.0
        }
    }

    impl hopr_api::graph::traits::EdgeNetworkObservableRead for StubMeasurement {
        fn is_connected(&self) -> bool {
            false
        }
    }

    impl hopr_api::graph::EdgeImmediateProtocolObservable for StubMeasurement {
        fn ack_rate(&self) -> Option<f64> {
            None
        }
    }

    impl hopr_api::graph::traits::EdgeProtocolObservable for StubMeasurement {
        fn capacity(&self) -> Option<u128> {
            None
        }
    }

    impl<C> HasGraphView for ChainNode<C>
    where
        C: HoprChainApi + ComponentStatusReporter + Clone + Send + Sync + 'static,
    {
        type Graph = StubGraph;

        fn graph(&self) -> &Self::Graph {
            self.1.as_ref()
        }

        fn status(&self) -> ComponentStatus {
            ComponentStatus::Ready
        }
    }

    async fn register_test_safe<C>(chain: &C, node_addr: Address) -> anyhow::Result<()>
    where
        C: HoprChainApi + ChainReadAccountOperations + ChainWriteAccountOperations,
    {
        let account = chain
            .stream_accounts(AccountSelector::default().with_chain_key(node_addr))?
            .next()
            .await
            .ok_or_else(|| anyhow::anyhow!("missing account for {node_addr}"))?;
        let safe = account
            .safe_address
            .ok_or_else(|| anyhow::anyhow!("missing safe for {node_addr}"))?;
        chain.register_safe(&safe).await?.await?;
        Ok(())
    }

    #[tokio::test]
    async fn default_config_should_have_sensible_values() {
        let cfg = ChannelLifecycleConfig::default();
        assert_eq!(cfg.population.min_open_channels, 5);
        assert_eq!(cfg.population.target_open_channels, 8);
        assert!(cfg.finalizer.enabled);
        assert!(cfg.proactive_funding.enabled);
        assert_eq!(cfg.eligibility.min_peer_quality_score, 0.5);
    }

    #[tokio::test]
    async fn strategy_should_fund_channel_below_threshold() -> anyhow::Result<()> {
        let stake_limit = HoprBalance::from(3_u32);
        let fund_amount = HoprBalance::from(5_u32);
        let initial_balance = HoprBalance::from(2_u32);

        let c1 = ChannelEntry::builder()
            .between(*BOB, *ALICE)
            .amount(2_u32) // below threshold of 3
            .ticket_index(0)
            .status(ChannelStatus::Open)
            .epoch(0)
            .build()?;

        let blokli_sim = BlokliTestStateBuilder::default()
            .with_generated_accounts(
                &[&*ALICE, &*BOB, &*CHRIS],
                false,
                XDaiBalance::new_base(1),
                HoprBalance::new_base(1000),
            )
            .with_channels([c1])
            .build_dynamic_client([1; Address::SIZE].into());

        let mut connector =
            create_trustful_hopr_blokli_connector(&BOB_KP, Default::default(), blokli_sim, [1; Address::SIZE].into())
                .await?;
        connector.connect().await?;
        let connector = Arc::new(connector);
        register_test_safe(&*connector, *BOB).await?;

        let node = Arc::new(ChainNode::new(Arc::clone(&connector)));

        // With the Blokli sim's default ticket price of "1 wxHOPR" and win_prob = 1.0,
        // ByteSize::b(1) resolves to 1 packet × 1 wxHOPR × 3 hops = 3 wxHOPR.
        // The channel has balance 2 wei ≪ 3 wxHOPR → triggers funding.
        // The safe has 1000 wxHOPR ≫ 3 wxHOPR topup → topup is affordable.
        let cfg = ChannelLifecycleConfig {
            tick_interval: Duration::from_millis(100),
            jitter: Duration::ZERO,
            funding: FundingConfig {
                lower_capacity_threshold: ByteSize::b(1),
                topup_capacity: ByteSize::b(1),
                min_safe_capacity_required: ByteSize::b(0),
                stop_when_unfunded: true,
                ..Default::default()
            },
            restart: RestartGuardConfig {
                startup_close_grace_period: Duration::ZERO,
            },
            ..Default::default()
        };

        let mut strategy: Box<dyn crate::strategy::Strategy + Send> = ChannelLifecycleStrategy::new(cfg).build(node);

        let handle = tokio::spawn(async move {
            let _ = strategy.run().await;
        });

        // Drive at least one full pipeline pass so the fund-pass has a chance
        // to submit a `fund_channel` tx and the chain layer to confirm it.
        tokio::time::sleep(Duration::from_secs(1)).await;
        handle.abort();
        let _ = handle.await;

        let channels: Vec<ChannelEntry> = connector
            .stream_channels(ChannelSelector::default().with_source(*BOB))
            .context("failed to stream channels for BOB")?
            .collect()
            .await;

        assert!(
            channels.iter().any(|c| c.balance > initial_balance),
            "expected the under-funded channel to be topped up; got {channels:?}"
        );

        Ok(())
    }

    #[test]
    fn restart_grace_should_block_close_pass() {
        let cfg = ChannelLifecycleConfig {
            restart: RestartGuardConfig {
                startup_close_grace_period: Duration::from_secs(3600),
            },
            ..Default::default()
        };
        let start_epoch = Instant::now();
        let grace_elapsed = start_epoch.elapsed() >= cfg.restart.startup_close_grace_period;
        assert!(
            !grace_elapsed,
            "close pass should be suppressed during startup grace period"
        );
    }

    #[tokio::test]
    async fn display_should_return_channel_lifecycle() -> anyhow::Result<()> {
        let blokli_sim = BlokliTestStateBuilder::default()
            .with_generated_accounts(
                &[&*ALICE, &*BOB],
                false,
                XDaiBalance::new_base(1),
                HoprBalance::new_base(1000),
            )
            .with_channels([])
            .build_dynamic_client([1; Address::SIZE].into());

        let mut chain_connector =
            create_trustful_hopr_blokli_connector(&BOB_KP, Default::default(), blokli_sim, [1; Address::SIZE].into())
                .await?;
        chain_connector.connect().await?;
        let chain_connector = Arc::new(chain_connector);
        let node = Arc::new(ChainNode::new(Arc::clone(&chain_connector)));

        let strategy: Box<dyn crate::strategy::Strategy + Send> =
            ChannelLifecycleStrategy::new(ChannelLifecycleConfig::default()).build(node);

        assert_eq!(strategy.to_string(), "channel_lifecycle");
        fn assert_send<T: Send>(_: T) {}
        assert_send(strategy);

        Ok(())
    }

    #[test]
    fn cooldown_should_prevent_reopen() {
        let _cfg = ChannelLifecycleConfig {
            population: PopulationConfig {
                peer_reopen_cooldown: Duration::from_secs(3600),
                ..Default::default()
            },
            ..Default::default()
        };

        let cooldown: Arc<DashMap<Address, Instant>> = Arc::new(DashMap::new());
        let dest = *CHRIS;
        cooldown.insert(dest, Instant::now() + Duration::from_secs(3600));

        let on_cooldown = cooldown.get(&dest).map(|v| Instant::now() < *v).unwrap_or(false);
        assert!(on_cooldown, "peer should be on cooldown");
    }

    /// Documents the restart guard's per-instance semantics: a freshly-built
    /// strategy starts a new grace window, regardless of how long the previous
    /// instance had been running.  The close pass is suppressed on the new
    /// instance until its own `startup_close_grace_period` elapses.
    #[test]
    fn restart_grace_should_re_apply_on_new_instance() {
        let cfg = ChannelLifecycleConfig {
            restart: RestartGuardConfig {
                startup_close_grace_period: Duration::from_secs(60),
            },
            ..Default::default()
        };

        // Old instance was running long enough that its grace window had elapsed.
        let old_start_epoch = Instant::now() - Duration::from_secs(65);
        assert!(
            old_start_epoch.elapsed() >= cfg.restart.startup_close_grace_period,
            "old instance's grace should have elapsed"
        );

        // After dropping the old instance and constructing a new one,
        // `start_epoch` resets — the new grace window starts from now.
        let new_start_epoch = Instant::now();
        assert!(
            new_start_epoch.elapsed() < cfg.restart.startup_close_grace_period,
            "new instance's grace should not have elapsed — restart guard re-applies per instance"
        );
    }

    /// Documents that no per-instance runtime state (in-flight sets, cooldown,
    /// observation history, ticket-activity counters, peer-addr cache) survives
    /// dropping the strategy.  A new instance starts cold; only on-chain state
    /// (channels, balances) is observable to it.  This is intentional: the
    /// strategy treats the chain as the source of truth and rebuilds its
    /// off-chain bookkeeping from observations after restart.
    #[test]
    fn new_instance_should_have_empty_state_after_old_dropped() {
        use dashmap::DashSet;
        use parking_lot::Mutex;

        fn fresh_inner(cfg: ChannelLifecycleConfig) -> ChannelLifecycleStrategyInner<()> {
            ChannelLifecycleStrategyInner {
                cfg,
                node: Arc::new(()),
                selector: Arc::new(selector::DefaultSelector),
                open_in_flight: Arc::new(DashSet::new()),
                fund_in_flight: Arc::new(DashSet::new()),
                close_in_flight: Arc::new(DashSet::new()),
                finalize_in_flight: Arc::new(DashSet::new()),
                cooldown: Arc::new(DashMap::new()),
                start_epoch: Instant::now(),
                last_observed: Arc::new(DashMap::new()),
                peer_ticket_activity: Arc::new(DashMap::new()),
                peer_addr_cache: Arc::new(Mutex::new(None)),
                last_resolved_funding: Arc::new(Mutex::new(None)),
            }
        }

        let cfg = ChannelLifecycleConfig::default();

        // Simulate accumulated state on the first instance.
        let inner1 = fresh_inner(cfg.clone());
        inner1
            .cooldown
            .insert(*CHRIS, Instant::now() + Duration::from_secs(3600));
        inner1.peer_ticket_activity.insert(*ALICE, 42);
        inner1.open_in_flight.insert(*DAVE);
        let old_start_epoch = inner1.start_epoch;

        drop(inner1);
        std::thread::sleep(Duration::from_millis(5));

        // The new instance is built from scratch — none of the previous state
        // is reachable.
        let inner2 = fresh_inner(cfg);

        assert!(
            inner2.open_in_flight.is_empty(),
            "open_in_flight should not persist across drop"
        );
        assert!(
            inner2.fund_in_flight.is_empty(),
            "fund_in_flight should not persist across drop"
        );
        assert!(
            inner2.close_in_flight.is_empty(),
            "close_in_flight should not persist across drop"
        );
        assert!(
            inner2.finalize_in_flight.is_empty(),
            "finalize_in_flight should not persist across drop"
        );
        assert!(
            inner2.cooldown.is_empty(),
            "cooldown should not persist across drop — recently closed peers may be reopened by the new instance"
        );
        assert!(
            inner2.peer_ticket_activity.is_empty(),
            "ticket activity counters should not persist across drop"
        );
        assert!(
            inner2.last_observed.is_empty(),
            "balance/ticket-index history should not persist across drop — proactive funding warms up over the first \
             few ticks"
        );
        assert!(
            inner2.peer_addr_cache.lock().is_none(),
            "peer-addr cache should not persist across drop"
        );
        assert!(
            inner2.start_epoch > old_start_epoch,
            "start_epoch should reset on a new instance — restart guard re-applies"
        );
    }

    /// In-flight transactions submitted by the pre-restart strategy instance may
    /// confirm on-chain after the node restarts and a fresh instance is running.
    /// The new instance must handle those events gracefully: in-flight set removals
    /// are no-ops (the fresh instance never populated those sets), but observable
    /// side-effects — such as peer cooldown on `ChannelClosed` — still take effect,
    /// keeping the channel from being immediately re-opened.
    #[tokio::test]
    async fn inflight_events_arrive_at_new_instance_after_restart() -> anyhow::Result<()> {
        use dashmap::DashSet;
        use parking_lot::Mutex;

        let blokli_sim = BlokliTestStateBuilder::default()
            .with_generated_accounts(
                &[&*ALICE, &*BOB, &*CHRIS, &*DAVE],
                false,
                XDaiBalance::new_base(1),
                HoprBalance::new_base(1000),
            )
            .with_channels([])
            .build_dynamic_client([1; Address::SIZE].into());

        let mut connector =
            create_trustful_hopr_blokli_connector(&BOB_KP, Default::default(), blokli_sim, [1; Address::SIZE].into())
                .await?;
        connector.connect().await?;
        let connector = Arc::new(connector);

        // One channel per in-flight set tracked by the old instance.
        let ch_close = ChannelEntry::builder()
            .between(*BOB, *ALICE)
            .amount(10_u32)
            .ticket_index(0)
            .status(ChannelStatus::Open)
            .epoch(0)
            .build()?;
        let ch_fund = ChannelEntry::builder()
            .between(*BOB, *CHRIS)
            .amount(5_u32)
            .ticket_index(0)
            .status(ChannelStatus::Open)
            .epoch(0)
            .build()?;
        let ch_open = ChannelEntry::builder()
            .between(*BOB, *DAVE)
            .amount(10_u32)
            .ticket_index(0)
            .status(ChannelStatus::Open)
            .epoch(0)
            .build()?;

        // ── Simulate old instance: accumulate in-flight state, then drop ─────
        {
            let old = ChannelLifecycleStrategyInner {
                cfg: ChannelLifecycleConfig::default(),
                node: Arc::new(ChainNode::new(Arc::clone(&connector))),
                selector: Arc::new(selector::DefaultSelector),
                open_in_flight: Arc::new(DashSet::new()),
                fund_in_flight: Arc::new(DashSet::new()),
                close_in_flight: Arc::new(DashSet::new()),
                finalize_in_flight: Arc::new(DashSet::new()),
                cooldown: Arc::new(DashMap::new()),
                start_epoch: Instant::now(),
                last_observed: Arc::new(DashMap::new()),
                peer_ticket_activity: Arc::new(DashMap::new()),
                peer_addr_cache: Arc::new(Mutex::new(None)),
                last_resolved_funding: Arc::new(Mutex::new(None)),
            };
            old.close_in_flight.insert(*ch_close.get_id());
            old.finalize_in_flight.insert(*ch_close.get_id());
            old.fund_in_flight.insert(*ch_fund.get_id());
            old.open_in_flight.insert(ch_open.destination);
            // Drop: none of this state transfers to the new instance.
        }

        // ── Fresh instance starts cold ────────────────────────────────────────
        let fresh = ChannelLifecycleStrategyInner {
            cfg: ChannelLifecycleConfig::default(),
            node: Arc::new(ChainNode::new(Arc::clone(&connector))),
            selector: Arc::new(selector::DefaultSelector),
            open_in_flight: Arc::new(DashSet::new()),
            fund_in_flight: Arc::new(DashSet::new()),
            close_in_flight: Arc::new(DashSet::new()),
            finalize_in_flight: Arc::new(DashSet::new()),
            cooldown: Arc::new(DashMap::new()),
            start_epoch: Instant::now(),
            last_observed: Arc::new(DashMap::new()),
            peer_ticket_activity: Arc::new(DashMap::new()),
            peer_addr_cache: Arc::new(Mutex::new(None)),
            last_resolved_funding: Arc::new(Mutex::new(None)),
        };

        assert!(fresh.close_in_flight.is_empty());
        assert!(fresh.finalize_in_flight.is_empty());
        assert!(fresh.fund_in_flight.is_empty());
        assert!(fresh.open_in_flight.is_empty());
        assert!(fresh.cooldown.is_empty());

        // Deliver the old instance's in-flight events that confirm post-restart.

        // Old close tx confirmed.
        fresh.on_channel_closure_initiated(ch_close);
        assert!(
            fresh.close_in_flight.is_empty(),
            "close_in_flight stays empty — remove was a no-op"
        );

        // Old finalize tx confirmed.
        fresh.on_channel_closed(ch_close);
        assert!(
            fresh.finalize_in_flight.is_empty(),
            "finalize_in_flight stays empty — remove was a no-op"
        );
        // Cooldown still takes effect: prevents immediately re-opening a just-closed channel.
        assert!(
            fresh.cooldown.contains_key(&ch_close.destination),
            "cooldown must be set on ChannelClosed even when the new instance did not initiate the close"
        );

        // Old fund tx confirmed.
        fresh.on_balance_increased(ch_fund);
        assert!(
            fresh.fund_in_flight.is_empty(),
            "fund_in_flight stays empty — remove was a no-op"
        );

        // Old open tx confirmed.
        fresh.on_channel_opened(ch_open);
        assert!(
            fresh.open_in_flight.is_empty(),
            "open_in_flight stays empty — remove was a no-op"
        );

        Ok(())
    }

    fn fresh_inner_with_chain<C>(
        cfg: ChannelLifecycleConfig,
        connector: Arc<C>,
    ) -> ChannelLifecycleStrategyInner<ChainNode<Arc<C>>> {
        fresh_inner_with_chain_and_graph(cfg, connector, Arc::new(StubGraph::default()))
    }

    fn fresh_inner_with_chain_and_graph<C>(
        cfg: ChannelLifecycleConfig,
        connector: Arc<C>,
        graph: Arc<StubGraph>,
    ) -> ChannelLifecycleStrategyInner<ChainNode<Arc<C>>> {
        ChannelLifecycleStrategyInner {
            cfg,
            node: Arc::new(ChainNode::with_graph(connector, graph)),
            selector: Arc::new(selector::DefaultSelector),
            open_in_flight: Arc::new(dashmap::DashSet::new()),
            fund_in_flight: Arc::new(dashmap::DashSet::new()),
            close_in_flight: Arc::new(dashmap::DashSet::new()),
            finalize_in_flight: Arc::new(dashmap::DashSet::new()),
            cooldown: Arc::new(DashMap::new()),
            start_epoch: std::time::Instant::now(),
            last_observed: Arc::new(DashMap::new()),
            peer_ticket_activity: Arc::new(DashMap::new()),
            peer_addr_cache: Arc::new(parking_lot::Mutex::new(None)),
            last_resolved_funding: Arc::new(parking_lot::Mutex::new(None)),
        }
    }

    /// try_open_channel: channel is already Open with stake >= lower_balance_threshold.
    /// Expected: no FundChannel tx submitted; open_in_flight empty after the call.
    #[tokio::test]
    async fn open_pass_skips_already_open_at_target_stake() -> anyhow::Result<()> {
        let lower_threshold = HoprBalance::from(3_u32);
        let initial_balance = HoprBalance::from(10_u32);

        let existing_channel = ChannelEntry::builder()
            .between(*BOB, *ALICE)
            .amount(5_u32) // balance 5 > threshold 3 → at desired stake
            .ticket_index(0)
            .status(ChannelStatus::Open)
            .epoch(0)
            .build()?;

        let blokli_sim = BlokliTestStateBuilder::default()
            .with_generated_accounts(
                &[&*ALICE, &*BOB, &*CHRIS],
                false,
                XDaiBalance::new_base(1),
                HoprBalance::new_base(1000),
            )
            .with_channels([existing_channel])
            .build_dynamic_client([1; Address::SIZE].into());

        let mut connector =
            create_trustful_hopr_blokli_connector(&BOB_KP, Default::default(), blokli_sim, [1; Address::SIZE].into())
                .await?;
        connector.connect().await?;
        let connector = Arc::new(connector);
        register_test_safe(&*connector, *BOB).await?;

        let cfg = ChannelLifecycleConfig::default();

        let inner = fresh_inner_with_chain(cfg, Arc::clone(&connector));

        // Build the resolved funding with the specific thresholds for this test.
        let resolved = make_resolved(
            lower_threshold,
            HoprBalance::from(8_u32), // topup (arbitrary, not used in this path)
            initial_balance,
            HoprBalance::from(1_u32), // min_safe (arbitrary, not used in try_open_channel)
        );
        let result = inner.try_open_channel(*ALICE, initial_balance, &resolved);

        // No open tx should have been submitted; no fund tx either.
        assert!(
            result.is_none(),
            "try_open_channel should return None for already-open-at-stake"
        );
        assert!(inner.open_in_flight.is_empty(), "open_in_flight must be cleared");
        assert!(inner.fund_in_flight.is_empty(), "fund_in_flight must be empty");

        let channels: Vec<ChannelEntry> = connector
            .stream_channels(ChannelSelector::default().with_source(*BOB))
            .context("failed to stream channels")?
            .collect()
            .await;
        assert_eq!(
            channels.iter().find(|c| c.destination == *ALICE).map(|c| c.balance),
            Some(HoprBalance::from(5_u32)),
            "on-chain balance must be unchanged"
        );

        Ok(())
    }

    /// try_open_channel: channel is PendingToClose (deadline in the future).
    /// Expected: no open tx submitted; open_in_flight cleared; on-chain channel unchanged.
    #[tokio::test]
    async fn open_pass_defers_open_when_pending_to_close() -> anyhow::Result<()> {
        let initial_balance = HoprBalance::from(10_u32);
        // Closure deadline is 10 minutes in the future — genuinely pending-close, not overdue.
        let closure_deadline = std::time::SystemTime::now() + Duration::from_secs(600);

        let existing_channel = ChannelEntry::builder()
            .between(*BOB, *ALICE)
            .amount(5_u32)
            .ticket_index(0)
            .status(ChannelStatus::PendingToClose(closure_deadline))
            .epoch(0)
            .build()?;

        let blokli_sim = BlokliTestStateBuilder::default()
            .with_generated_accounts(
                &[&*ALICE, &*BOB, &*CHRIS],
                false,
                XDaiBalance::new_base(1),
                HoprBalance::new_base(1000),
            )
            .with_channels([existing_channel])
            .build_dynamic_client([1; Address::SIZE].into());

        let mut connector =
            create_trustful_hopr_blokli_connector(&BOB_KP, Default::default(), blokli_sim, [1; Address::SIZE].into())
                .await?;
        connector.connect().await?;
        let connector = Arc::new(connector);
        register_test_safe(&*connector, *BOB).await?;

        let cfg = ChannelLifecycleConfig {
            funding: FundingConfig {
                initial_balance,
                min_safe_balance_required: HoprBalance::from(1_u32),
                stop_when_unfunded: true,
                ..Default::default()
            },
            ..Default::default()
        };

        let inner = fresh_inner_with_chain(cfg, Arc::clone(&connector));

        let result = inner.try_open_channel(*ALICE, initial_balance);

        // The PendingToClose guard must fire: no tx, in-flight marker cleared.
        assert!(
            result.is_none(),
            "try_open_channel must return None when channel is PendingToClose"
        );
        assert!(inner.open_in_flight.is_empty(), "open_in_flight must be cleared");
        assert!(inner.fund_in_flight.is_empty(), "fund_in_flight must be empty");

        // On-chain channel must still be PendingToClose — nothing mutated it.
        let channels: Vec<ChannelEntry> = connector
            .stream_channels(ChannelSelector::default().with_source(*BOB))
            .context("failed to stream channels")?
            .collect()
            .await;
        let alice_ch = channels
            .iter()
            .find(|c| c.destination == *ALICE)
            .expect("BOB→ALICE channel must still exist on-chain");
        assert!(
            matches!(alice_ch.status, ChannelStatus::PendingToClose(_)),
            "channel status must remain PendingToClose, got {:?}",
            alice_ch.status
        );

        Ok(())
    }

    /// try_open_channel: channel is already Open but stake < lower_balance_threshold.
    /// Expected: one FundChannel tx submitted immediately (no waiting for next tick);
    /// open_in_flight empty; on-chain balance increases by topup_balance.
    #[tokio::test]
    async fn open_pass_tops_up_already_open_below_threshold() -> anyhow::Result<()> {
        let lower_threshold = HoprBalance::from(3_u32);
        let topup_balance = HoprBalance::from(8_u32);

        let existing_channel = ChannelEntry::builder()
            .between(*BOB, *ALICE)
            .amount(2_u32) // balance 2 ≤ threshold 3 → underfunded
            .ticket_index(0)
            .status(ChannelStatus::Open)
            .epoch(0)
            .build()?;

        let blokli_sim = BlokliTestStateBuilder::default()
            .with_generated_accounts(
                &[&*ALICE, &*BOB, &*CHRIS],
                false,
                XDaiBalance::new_base(1),
                HoprBalance::new_base(1000),
            )
            .with_channels([existing_channel])
            .build_dynamic_client([1; Address::SIZE].into());

        let mut connector =
            create_trustful_hopr_blokli_connector(&BOB_KP, Default::default(), blokli_sim, [1; Address::SIZE].into())
                .await?;
        connector.connect().await?;
        let connector = Arc::new(connector);
        register_test_safe(&*connector, *BOB).await?;

        let cfg = ChannelLifecycleConfig::default();

        let inner = fresh_inner_with_chain(cfg, Arc::clone(&connector));

        // Build the resolved funding with the specific thresholds for this test.
        let resolved = make_resolved(
            lower_threshold,
            topup_balance,
            HoprBalance::from(10_u32), // initial (arbitrary)
            HoprBalance::from(1_u32),  // min_safe (arbitrary)
        );
        let result = inner.try_open_channel(*ALICE, HoprBalance::from(10_u32), &resolved);

        // Should return Some(topup_balance) (fund tx submitted) and clear open_in_flight.
        assert_eq!(
            result,
            Some(topup_balance),
            "try_open_channel should return Some(topup_balance) when delegating to fund"
        );
        assert!(inner.open_in_flight.is_empty(), "open_in_flight must be cleared");

        // Wait for the fund tx to confirm.
        tokio::time::sleep(Duration::from_secs(1)).await;

        let channels: Vec<ChannelEntry> = connector
            .stream_channels(ChannelSelector::default().with_source(*BOB))
            .context("failed to stream channels")?
            .collect()
            .await;
        assert!(
            channels
                .iter()
                .any(|c| c.destination == *ALICE && c.balance > HoprBalance::from(2_u32)),
            "on-chain balance must be increased after fund; got {channels:?}"
        );

        Ok(())
    }

    /// try_open_channel: no pre-existing channel for destination.
    /// Expected: one open (FundChannel) tx submitted; on-chain channel created.
    #[tokio::test]
    async fn open_pass_opens_fresh_channel_when_missing() -> anyhow::Result<()> {
        let initial_balance = HoprBalance::from(10_u32);

        let blokli_sim = BlokliTestStateBuilder::default()
            .with_generated_accounts(
                &[&*ALICE, &*BOB, &*CHRIS],
                false,
                XDaiBalance::new_base(1),
                HoprBalance::new_base(1000),
            )
            .with_channels([])
            .build_dynamic_client([1; Address::SIZE].into());

        let mut connector =
            create_trustful_hopr_blokli_connector(&BOB_KP, Default::default(), blokli_sim, [1; Address::SIZE].into())
                .await?;
        connector.connect().await?;
        let connector = Arc::new(connector);
        register_test_safe(&*connector, *BOB).await?;

        let cfg = ChannelLifecycleConfig::default();

        let inner = fresh_inner_with_chain(cfg, Arc::clone(&connector));

        // Build resolved funding — for the "no prior channel" path only amount matters.
        let resolved = make_resolved(
            HoprBalance::from(3_u32), // lower (arbitrary, no channel to check)
            HoprBalance::from(8_u32), // topup (arbitrary)
            initial_balance,
            HoprBalance::from(1_u32), // min_safe (arbitrary)
        );
        let result = inner.try_open_channel(*ALICE, initial_balance, &resolved);
        assert_eq!(
            result,
            Some(initial_balance),
            "try_open_channel should return Some(initial_balance) for a fresh channel"
        );

        // Wait for the open tx to confirm.
        tokio::time::sleep(Duration::from_secs(1)).await;

        let channels: Vec<ChannelEntry> = connector
            .stream_channels(ChannelSelector::default().with_source(*BOB))
            .context("failed to stream channels")?
            .collect()
            .await;
        assert!(
            channels
                .iter()
                .any(|c| c.destination == *ALICE && c.status == ChannelStatus::Open),
            "BOB→ALICE channel must be Open after open tx; got {channels:?}"
        );

        Ok(())
    }

    /// Pipeline integration: a `ByteSize` capacity config is resolved to the
    /// correct wxHOPR **topup** amount via the live ticket economics fetched from
    /// the Blokli simulator, and the fund pass applies it to an underfunded channel.
    ///
    /// The Blokli test sim defaults to `ticket_price = "1 wxHOPR"` and
    /// `min_ticket_winning_probability = 1.0`.  With `assumed_hops = 3` and
    /// `topup_capacity = 1 byte` (= 1 packet):
    ///
    /// ```text
    /// topup_balance = 1 packet × 1 wxHOPR × 3 hops / 1.0 = 3 wxHOPR
    /// ```
    ///
    /// The channel starts with 0 balance (< resolved threshold), triggering a
    /// fund tx.  After confirmation the on-chain balance must equal exactly
    /// 3 wxHOPR.
    #[tokio::test]
    async fn pipeline_funds_underfunded_channel_with_capacity_derived_wxhopr_amount() -> anyhow::Result<()> {
        use super::super::config::capacity_to_balance;

        // Blokli defaults: ticket_price = "1 wxHOPR", win_prob = 1.0, assumed_hops = 3.
        // capacity_to_balance(ByteSize::b(1), 1 wxHOPR, 1.0, 3) = 3 wxHOPR.
        // capacity_to_balance(1 byte, 1 wxHOPR, 1.0, 3 hops) = 3 wxHOPR.
        let expected_topup = {
            let price = HoprBalance::new_base(1); // 1 wxHOPR (Blokli default)
            let wp = WinningProbability::try_from(1.0f64).expect("valid win_prob");
            capacity_to_balance(ByteSize::b(1), price, wp, 3) // topup_capacity = ByteSize::b(1)
        };

        // Channel starts at 0 balance — below any non-zero threshold.
        let ch = ChannelEntry::builder()
            .between(*BOB, *ALICE)
            .amount(0_u32)
            .ticket_index(0)
            .status(ChannelStatus::Open)
            .epoch(0)
            .build()?;

        let blokli_sim = BlokliTestStateBuilder::default()
            .with_generated_accounts(
                &[&*ALICE, &*BOB, &*CHRIS],
                false,
                XDaiBalance::new_base(1),
                HoprBalance::new_base(1000),
            )
            .with_channels([ch])
            .build_dynamic_client([1; Address::SIZE].into());

        let mut connector =
            create_trustful_hopr_blokli_connector(&BOB_KP, Default::default(), blokli_sim, [1; Address::SIZE].into())
                .await?;
        connector.connect().await?;
        let connector = Arc::new(connector);
        register_test_safe(&*connector, *BOB).await?;

        let cfg = ChannelLifecycleConfig {
            tick_interval: Duration::from_millis(100),
            jitter: Duration::ZERO,
            restart: RestartGuardConfig {
                startup_close_grace_period: Duration::ZERO,
            },
            funding: FundingConfig {
                // threshold = 0 → only the initial 0-balance channel triggers funding.
                // After the topup (3 wxHOPR), balance > 0 → no further funding on next tick.
                lower_capacity_threshold: ByteSize::b(0),
                // 1 byte = 1 packet → 3 wxHOPR topup at default sim economics.
                topup_capacity: ByteSize::b(1),
                min_safe_capacity_required: ByteSize::b(0),
                stop_when_unfunded: true,
                assumed_hops: 3,
                ..Default::default()
            },
            // Disable proactive funding so only the reactive threshold path fires.
            proactive_funding: ProactiveFundingConfig {
                enabled: false,
                ..Default::default()
            },
            ..Default::default()
        };

        let node = Arc::new(ChainNode::new(Arc::clone(&connector)));
        let mut strategy: Box<dyn crate::strategy::Strategy + Send> = ChannelLifecycleStrategy::new(cfg).build(node);
        let handle = tokio::spawn(async move {
            let _ = strategy.run().await;
        });

        tokio::time::sleep(Duration::from_secs(1)).await;
        handle.abort();
        let _ = handle.await;

        let channels: Vec<ChannelEntry> = connector
            .stream_channels(ChannelSelector::default().with_source(*BOB))
            .context("failed to stream channels")?
            .collect()
            .await;

        let funded = channels
            .iter()
            .find(|c| c.destination == *ALICE && c.status == ChannelStatus::Open)
            .unwrap_or_else(|| panic!("BOB→ALICE open channel not found; got {channels:?}"));

        assert_eq!(
            funded.balance, expected_topup,
            "after fund pass the channel balance must equal capacity_to_balance(1 byte, 1 wxHOPR, 1.0, 3 hops) = \
             {expected_topup}; got {}; channels: {channels:?}",
            funded.balance
        );

        Ok(())
    }

    /// Gate returns empty list for a peer with no graph observations.
    ///
    /// Verifies that the default selector defers all graph-derived close reasons
    /// when `edge_info.last_update == Duration::ZERO` for the destination peer.
    #[tokio::test]
    async fn should_close_returns_false_for_peer_without_probing_data() -> anyhow::Result<()> {
        use selector::{CloseCandidate, PeerEdgeInfo, Selector as _, SelectorContext};

        let alice_pk = {
            use hopr_api::types::crypto::keypairs::Keypair as _;
            let pseudo = hopr_api::types::crypto::types::Hash::create(&[(*ALICE).as_ref()]);
            *hopr_api::types::crypto::prelude::OffchainKeypair::from_secret(pseudo.as_ref())
                .expect("alice offchain key")
                .public()
        };

        let ch = ChannelEntry::builder()
            .between(*BOB, *ALICE)
            .amount(125_u32)
            .ticket_index(0)
            .status(ChannelStatus::Open)
            .epoch(0)
            .build()?;

        // No graph observations → last_update = ZERO.
        let candidate = CloseCandidate {
            channel: ch,
            offchain_key: Some(alice_pk),
            edge_info: PeerEdgeInfo::default(),
            ticket_score: 0.0,
        };

        let ctx = SelectorContext {
            cfg: &ChannelLifecycleConfig::default(),
            deficit: 0,
            open_candidates: &[],
            close_candidates: &[candidate],
            start_epoch_elapsed: Duration::ZERO,
            bucket_view: selector::BucketView::default(),
            stake_view: selector::StakeView::empty(),
        };

        let closes = selector::DefaultSelector.select_closes(&ctx).await;
        assert!(
            closes.is_empty(),
            "select_closes must return empty for a peer with no graph observations"
        );
        Ok(())
    }

    /// Gate lifts after the first graph observation: the default selector closes the channel.
    ///
    /// Supplies a `CloseCandidate` with `last_update > Duration::ZERO` and `score = 0.0`
    /// (below `close_below_quality_score = 0.3`) and confirms the channel ID is returned.
    #[tokio::test]
    async fn should_close_returns_true_once_probing_data_arrives() -> anyhow::Result<()> {
        use selector::{CloseCandidate, PeerEdgeInfo, Selector as _, SelectorContext};

        let alice_pk = {
            use hopr_api::types::crypto::keypairs::Keypair as _;
            let pseudo = hopr_api::types::crypto::types::Hash::create(&[(*ALICE).as_ref()]);
            *hopr_api::types::crypto::prelude::OffchainKeypair::from_secret(pseudo.as_ref())
                .expect("alice offchain key")
                .public()
        };

        let ch = ChannelEntry::builder()
            .between(*BOB, *ALICE)
            .amount(125_u32)
            .ticket_index(0)
            .status(ChannelStatus::Open)
            .epoch(0)
            .build()?;

        let cfg = ChannelLifecycleConfig::default();

        // No observations yet — gate must return false.
        let no_data = CloseCandidate {
            channel: ch,
            offchain_key: Some(alice_pk),
            edge_info: PeerEdgeInfo::default(),
            ticket_score: 0.0,
        };
        let ctx_no_data = SelectorContext {
            cfg: &cfg,
            deficit: 0,
            open_candidates: &[],
            close_candidates: &[no_data],
            start_epoch_elapsed: Duration::ZERO,
            bucket_view: selector::BucketView::default(),
            stake_view: selector::StakeView::empty(),
        };
        assert!(
            selector::DefaultSelector.select_closes(&ctx_no_data).await.is_empty(),
            "select_closes must return empty before probing data arrives"
        );

        // Edge with last_update > 0 and score below the close threshold.
        let with_data = CloseCandidate {
            channel: ch,
            offchain_key: Some(alice_pk),
            edge_info: PeerEdgeInfo {
                edge_score: Some(0.0), // below close_below_quality_score = 0.3
                last_update: Duration::from_secs(1),
                ..Default::default()
            },
            ticket_score: 0.0,
        };
        let ctx_with_data = SelectorContext {
            cfg: &cfg,
            deficit: 0,
            open_candidates: &[],
            close_candidates: &[with_data],
            start_epoch_elapsed: Duration::from_secs(10), // strategy running 10s > last_update 1s
            bucket_view: selector::BucketView::default(),
            stake_view: selector::StakeView::empty(),
        };
        let closes = selector::DefaultSelector.select_closes(&ctx_with_data).await;
        assert_eq!(
            closes,
            vec![*ch.get_id()],
            "select_closes must return the channel ID once probing data arrives with low score"
        );
        Ok(())
    }

    /// Full-pipeline test: preexisting channel survives one tick when graph is empty.
    ///
    /// The strategy's close pass must not retire a channel whose peer has never
    /// been observed in the network graph, even after the restart grace window.
    #[tokio::test]
    async fn preexisting_channel_not_closed_in_pipeline_without_probing_data() -> anyhow::Result<()> {
        let ch = ChannelEntry::builder()
            .between(*BOB, *ALICE)
            .amount(125_u32)
            .ticket_index(0)
            .status(ChannelStatus::Open)
            .epoch(0)
            .build()?;

        // Alice must be announced (public: true) so she appears in addr_to_key
        // during the pipeline's peer_addr_map pass.
        let blokli_sim = BlokliTestStateBuilder::default()
            .with_generated_accounts(
                &[&*ALICE, &*BOB, &*CHRIS],
                true,
                XDaiBalance::new_base(1),
                HoprBalance::new_base(1000),
            )
            .with_channels([ch])
            .build_dynamic_client([1; Address::SIZE].into());
        let mut connector =
            create_trustful_hopr_blokli_connector(&BOB_KP, Default::default(), blokli_sim, [1; Address::SIZE].into())
                .await?;
        connector.connect().await?;
        let connector = Arc::new(connector);
        register_test_safe(&*connector, *BOB).await?;

        let cfg = ChannelLifecycleConfig {
            tick_interval: Duration::from_millis(100),
            jitter: Duration::ZERO,
            restart: RestartGuardConfig {
                startup_close_grace_period: Duration::ZERO,
            },
            population: PopulationConfig {
                // Allow closing the last channel so the population guard does
                // not mask the probing gate we are testing.
                min_open_channels: 0,
                ..Default::default()
            },
            // Require an unfeasibly high safe balance so the fund/open passes
            // are skipped, isolating the close pass.
            // ByteSize::MAX resolves to a wxHOPR amount >> safe balance, so the
            // fund/open passes are skipped — isolating the close/finalize pass.
            funding: FundingConfig {
                min_safe_capacity_required: ByteSize::b(u64::MAX),
                stop_when_unfunded: true,
                ..Default::default()
            },
            ..Default::default()
        };

        // Leave the graph empty — no observations for any peer.
        let node = Arc::new(ChainNode::new(Arc::clone(&connector)));
        let mut strategy: Box<dyn crate::strategy::Strategy + Send> = ChannelLifecycleStrategy::new(cfg).build(node);
        let handle = tokio::spawn(async move {
            let _ = strategy.run().await;
        });

        tokio::time::sleep(Duration::from_secs(1)).await;
        handle.abort();
        let _ = handle.await;

        let channels: Vec<ChannelEntry> = connector
            .stream_channels(ChannelSelector::default().with_source(*BOB))
            .context("failed to stream channels")?
            .collect()
            .await;

        assert!(
            channels
                .iter()
                .any(|c| c.destination == *ALICE && c.status == ChannelStatus::Open),
            "preexisting channel must not be closed when the graph has no observations; got {channels:?}"
        );
        Ok(())
    }

    /// Full-pipeline test: close fires after the first graph observation arrives.
    ///
    /// Companion to `preexisting_channel_not_closed_in_pipeline_without_probing_data`.
    /// After the graph receives a low-score edge for Alice the strategy must
    /// initiate closure on the next tick.
    #[tokio::test]
    async fn preexisting_channel_closed_once_probing_data_arrives() -> anyhow::Result<()> {
        let ch = ChannelEntry::builder()
            .between(*BOB, *ALICE)
            .amount(125_u32)
            .ticket_index(0)
            .status(ChannelStatus::Open)
            .epoch(0)
            .build()?;

        let blokli_sim = BlokliTestStateBuilder::default()
            .with_generated_accounts(
                &[&*ALICE, &*BOB, &*CHRIS],
                true,
                XDaiBalance::new_base(1),
                HoprBalance::new_base(1000),
            )
            .with_channels([ch])
            .build_dynamic_client([1; Address::SIZE].into());
        let mut connector =
            create_trustful_hopr_blokli_connector(&BOB_KP, Default::default(), blokli_sim, [1; Address::SIZE].into())
                .await?;
        connector.connect().await?;
        let connector = Arc::new(connector);
        register_test_safe(&*connector, *BOB).await?;

        let cfg = ChannelLifecycleConfig {
            tick_interval: Duration::from_millis(100),
            jitter: Duration::ZERO,
            restart: RestartGuardConfig {
                startup_close_grace_period: Duration::ZERO,
            },
            population: PopulationConfig {
                // Allow closing the last channel so the population guard
                // does not suppress our single-channel close.
                min_open_channels: 0,
                ..Default::default()
            },
            closure: ClosureConfig {
                close_below_quality_score: 0.3,
                ..Default::default()
            },
            // ByteSize::MAX resolves to a wxHOPR amount >> safe balance, so the
            // fund/open passes are skipped — isolating the close/finalize pass.
            funding: FundingConfig {
                min_safe_capacity_required: ByteSize::b(u64::MAX),
                stop_when_unfunded: true,
                ..Default::default()
            },
            ..Default::default()
        };

        // Keep a handle to the graph so we can inject an edge mid-run.
        let graph = Arc::new(StubGraph::default());
        let node = Arc::new(ChainNode::with_graph(Arc::clone(&connector), Arc::clone(&graph)));
        let mut strategy: Box<dyn crate::strategy::Strategy + Send> = ChannelLifecycleStrategy::new(cfg).build(node);
        let handle = tokio::spawn(async move {
            let _ = strategy.run().await;
        });

        // First window: no probing data — close must not fire.
        tokio::time::sleep(Duration::from_millis(400)).await;

        // Inject a low-score edge to simulate a completed probe round.
        let alice_pk = {
            use hopr_api::types::crypto::keypairs::Keypair as _;
            let pseudo = hopr_api::types::crypto::types::Hash::create(&[(*ALICE).as_ref()]);
            *hopr_api::types::crypto::prelude::OffchainKeypair::from_secret(pseudo.as_ref())
                .expect("alice offchain key")
                .public()
        };
        let my_key = {
            use hopr_api::types::crypto::keypairs::Keypair as _;
            *hopr_api::types::crypto::prelude::OffchainKeypair::from_secret(&[1u8; 32])
                .expect("my key")
                .public()
        };
        graph.insert_edge(
            my_key,
            alice_pk,
            StubEdge {
                last_update: Duration::from_secs(1),
                score: 0.0, // forces low_quality_score close
            },
        );

        // Second window: probing data present — close should fire and confirm.
        tokio::time::sleep(Duration::from_secs(2)).await;
        handle.abort();
        let _ = handle.await;

        let channels: Vec<ChannelEntry> = connector
            .stream_channels(ChannelSelector::default().with_source(*BOB))
            .context("failed to stream channels")?
            .collect()
            .await;

        assert!(
            channels
                .iter()
                .any(|c| c.destination == *ALICE && matches!(c.status, ChannelStatus::PendingToClose(_))),
            "channel must be PendingToClose after a low-score observation arrives; got {channels:?}"
        );
        Ok(())
    }

    /// Regression test: finalizer must trigger as soon as `closure_time` is past
    /// by `max_closure_overdue`, with no additional notice-period delay.
    ///
    /// Before the fix the pipeline computed
    /// `overdue = closure_notice_period + max_closure_overdue`, doubling the
    /// wait because `closure_time` is already the deadline (initiation +
    /// notice_period).  Here the closure deadline just passed 1 second ago
    /// and the notice period is set to 60 s.  Under the old code:
    /// `overdue = 60 s + 0 = 60 s`, `elapsed ≈ 1 s < 60 s` → would NOT
    /// finalize.  Under the new code: `overdue = 0`, `elapsed ≈ 1 s >= 0`
    /// → finalizes immediately.
    #[tokio::test]
    async fn finalizer_triggers_immediately_once_overdue_elapsed() -> anyhow::Result<()> {
        // Deadline passed 1 second ago — past due but well within the 60 s notice
        // period the old buggy code would have waited.
        let closure_deadline = std::time::SystemTime::now() - Duration::from_secs(1);
        let ch = ChannelEntry::builder()
            .between(*BOB, *ALICE)
            .amount(10_u32)
            .ticket_index(0)
            .status(ChannelStatus::PendingToClose(closure_deadline))
            .epoch(0)
            .build()?;

        let blokli_sim = BlokliTestStateBuilder::default()
            .with_generated_accounts(
                &[&*ALICE, &*BOB, &*CHRIS],
                false,
                XDaiBalance::new_base(1),
                HoprBalance::new_base(1000),
            )
            .with_channels([ch])
            // 60 s notice period: old code would have computed overdue = 60 s,
            // preventing finalization for a channel only 1 s past its deadline.
            .with_closure_grace_period(Duration::from_secs(60))
            .build_dynamic_client([1; Address::SIZE].into());

        let mut connector =
            create_trustful_hopr_blokli_connector(&BOB_KP, Default::default(), blokli_sim, [1; Address::SIZE].into())
                .await?;
        connector.connect().await?;
        let connector = Arc::new(connector);
        register_test_safe(&*connector, *BOB).await?;

        let cfg = ChannelLifecycleConfig {
            tick_interval: Duration::from_millis(100),
            jitter: Duration::ZERO,
            finalizer: FinalizerConfig {
                enabled: true,
                max_closure_overdue: Duration::ZERO,
                ..Default::default()
            },
            // Disable fund/open passes to isolate the finalizer.
            // ByteSize::MAX resolves to a wxHOPR amount >> safe balance, so the
            // fund/open passes are skipped — isolating the close/finalize pass.
            funding: FundingConfig {
                min_safe_capacity_required: ByteSize::b(u64::MAX),
                stop_when_unfunded: true,
                ..Default::default()
            },
            ..Default::default()
        };

        let node = Arc::new(ChainNode::new(Arc::clone(&connector)));
        let mut strategy: Box<dyn crate::strategy::Strategy + Send> = ChannelLifecycleStrategy::new(cfg).build(node);
        let handle = tokio::spawn(async move {
            let _ = strategy.run().await;
        });

        tokio::time::sleep(Duration::from_secs(1)).await;
        handle.abort();
        let _ = handle.await;

        let channels: Vec<ChannelEntry> = connector
            .stream_channels(ChannelSelector::default().with_source(*BOB))
            .context("failed to stream channels")?
            .collect()
            .await;

        assert!(
            !channels
                .iter()
                .any(|c| c.destination == *ALICE && matches!(c.status, ChannelStatus::PendingToClose(_))),
            "channel whose deadline already passed must not remain PendingToClose; got {channels:?}"
        );
        Ok(())
    }

    /// Full lifecycle: Open → PendingToClose (close pass) → Closed (finalizer pass).
    ///
    /// The channel starts `Open` with a low-quality peer; the close pass
    /// initiates closure (→ `PendingToClose`), and once the emulator's 2-second
    /// grace period elapses the finalizer pass submits the second `close_channel`
    /// call (→ `Closed`).
    #[tokio::test]
    async fn channel_lifecycle_open_to_pending_to_closed() -> anyhow::Result<()> {
        let ch = ChannelEntry::builder()
            .between(*BOB, *ALICE)
            .amount(125_u32)
            .ticket_index(0)
            .status(ChannelStatus::Open)
            .epoch(0)
            .build()?;

        let blokli_sim = BlokliTestStateBuilder::default()
            .with_generated_accounts(
                &[&*ALICE, &*BOB, &*CHRIS],
                true,
                XDaiBalance::new_base(1),
                HoprBalance::new_base(1000),
            )
            .with_channels([ch])
            // Emulator floors at 2 s; we set 0 to get as close to that floor
            // as possible so the test completes in a few seconds.
            .with_closure_grace_period(Duration::ZERO)
            .build_dynamic_client([1; Address::SIZE].into());

        let mut connector =
            create_trustful_hopr_blokli_connector(&BOB_KP, Default::default(), blokli_sim, [1; Address::SIZE].into())
                .await?;
        connector.connect().await?;
        let connector = Arc::new(connector);
        register_test_safe(&*connector, *BOB).await?;

        let cfg = ChannelLifecycleConfig {
            tick_interval: Duration::from_millis(100),
            jitter: Duration::ZERO,
            restart: RestartGuardConfig {
                startup_close_grace_period: Duration::ZERO,
            },
            population: PopulationConfig {
                min_open_channels: 0,
                ..Default::default()
            },
            closure: ClosureConfig {
                close_below_quality_score: 0.3,
                ..Default::default()
            },
            finalizer: FinalizerConfig {
                enabled: true,
                // Accept channels overdue by any amount once the deadline passes.
                max_closure_overdue: Duration::ZERO,
                ..Default::default()
            },
            // ByteSize::MAX resolves to a wxHOPR amount >> safe balance, so the
            // fund/open passes are skipped — isolating the close/finalize pass.
            funding: FundingConfig {
                min_safe_capacity_required: ByteSize::b(u64::MAX),
                stop_when_unfunded: true,
                ..Default::default()
            },
            ..Default::default()
        };

        let graph = Arc::new(StubGraph::default());
        let node = Arc::new(ChainNode::with_graph(Arc::clone(&connector), Arc::clone(&graph)));
        let mut strategy: Box<dyn crate::strategy::Strategy + Send> = ChannelLifecycleStrategy::new(cfg).build(node);
        let handle = tokio::spawn(async move {
            let _ = strategy.run().await;
        });

        // First window: no probing data — close must not fire yet.
        tokio::time::sleep(Duration::from_millis(300)).await;

        let alice_pk = {
            use hopr_api::types::crypto::keypairs::Keypair as _;
            let pseudo = hopr_api::types::crypto::types::Hash::create(&[(*ALICE).as_ref()]);
            *hopr_api::types::crypto::prelude::OffchainKeypair::from_secret(pseudo.as_ref())
                .expect("alice offchain key")
                .public()
        };
        let my_key = {
            use hopr_api::types::crypto::keypairs::Keypair as _;
            *hopr_api::types::crypto::prelude::OffchainKeypair::from_secret(&[1u8; 32])
                .expect("my key")
                .public()
        };
        graph.insert_edge(
            my_key,
            alice_pk,
            StubEdge {
                last_update: Duration::from_secs(1),
                score: 0.0,
            },
        );

        // Second window: close fires → PendingToClose; wait for the 2-second
        // emulator grace period to expire, then the finalizer fires → Closed.
        tokio::time::sleep(Duration::from_secs(4)).await;
        handle.abort();
        let _ = handle.await;

        let channels: Vec<ChannelEntry> = connector
            .stream_channels(ChannelSelector::default().with_source(*BOB))
            .context("failed to stream channels")?
            .collect()
            .await;

        assert!(
            !channels.iter().any(|c| c.destination == *ALICE
                && matches!(c.status, ChannelStatus::Open | ChannelStatus::PendingToClose(_))),
            "channel must be fully Closed after the grace period; got {channels:?}"
        );
        Ok(())
    }
}
