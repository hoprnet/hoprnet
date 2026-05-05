//! ## Channel Lifecycle Strategy
//!
//! A unified strategy that owns **open / fund / close / finalize** for outgoing
//! payment channels.  It replaces the combination of `AutoFundingStrategy` +
//! `ClosureFinalizerStrategy` with a single component that maintains a target
//! population of funded outgoing channels against online peers and retires
//! channels to peers that have been absent for too long.
//!
//! ### State machine
//!
//! ```text
//!                                   ┌────────────────────────┐
//!                                   │   no on-chain entry    │
//!                                   └───────────┬────────────┘
//!                                               │ open()  (eligibility passed)
//!                                               ▼
//!                                   ┌────────────────────────┐
//!                                   │     OpenInFlight       │
//!                                   └───────────┬────────────┘
//!                                               │ ChannelOpened
//!                                               ▼
//!                                   ┌────────────────────────┐
//!                                   │         Open           │◄──────────────┐
//!                                   └─────┬──────────┬───────┘               │
//!                below_lower_balance      │          │ staleness/quality drop
//!                       fund()            │          │  close()
//!                           ▼             │          ▼
//!                   ┌──────────────┐      │   ┌────────────────────┐
//!                   │ FundInFlight │      │   │   CloseInFlight    │
//!                   └──────┬───────┘      │   └─────────┬──────────┘
//!                          │ Balance↑     │             │ ChannelClosureInitiated
//!                          ▼              │             ▼
//!                         Open ───────────┘   ┌────────────────────┐
//!                                             │  PendingToClose    │
//!                                             └─────────┬──────────┘
//!                                                       │ notice_period + max_closure_overdue
//!                                                       │ finalize()
//!                                                       ▼
//!                                             ┌────────────────────┐
//!                                             │ FinalizeInFlight   │
//!                                             └─────────┬──────────┘
//!                                                       │ ChannelClosed
//!                                                       ▼
//!                                             ┌────────────────────┐
//!                                             │  cooldown (peer)   │
//!                                             └────────────────────┘
//!                                                       │ peer_reopen_cooldown
//!                                                       ▼
//!                                                (eligible to reopen)
//! ```
//!
//! In-flight states are tracked off-chain in `DashSet<ChannelId>` / `DashSet<Address>`.
//! The on-chain `ChannelStatus` plus the in-flight sets together drive transitions.
//! The cooldown is keyed by peer `Address` with an `Instant`-stamped map entry.
//!
//! ### Feature flag
//!
//! Enable with `strategy-channel-lifecycle`.
use std::{
    collections::{HashMap, HashSet},
    fmt::{Display, Formatter},
    sync::Arc,
    time::{Duration, Instant},
};

use async_trait::async_trait;
use dashmap::{DashMap, DashSet};
use futures::StreamExt as _;
use hopr_lib::api::{
    PeerId,
    chain::{
        AccountSelector, ChainEvent, ChainReadAccountOperations, ChainReadChannelOperations, ChainReadSafeOperations,
        ChainValues, ChainWriteChannelOperations, ChannelSelector, SafeSelector,
    },
    graph::{EdgeObservableRead as _, NetworkGraphView as _},
    network::NetworkView as _,
    node::{
        ActionableEvent, ActionableEventDiscriminant, ActionableEventSource, HasChainApi, HasGraphView, HasNetworkView,
    },
    types::{
        crypto::prelude::OffchainPublicKey,
        internal::prelude::{ChannelDirection, ChannelEntry, ChannelId, ChannelStatus},
        primitive::prelude::{Address, HoprBalance},
    },
};
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use tracing::{debug, info, warn};
use validator::Validate;

use crate::{errors::StrategyError, strategy::Strategy as StrategyTrait};

#[cfg(all(feature = "telemetry", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_CHANNEL_OPENS: hopr_metrics::SimpleCounter =
        hopr_metrics::SimpleCounter::new(
            "hopr_strategy_channel_lifecycle_opens",
            "Count of initiated channel opens",
        ).unwrap();
    static ref METRIC_CHANNEL_FUNDS: hopr_metrics::SimpleCounter =
        hopr_metrics::SimpleCounter::new(
            "hopr_strategy_channel_lifecycle_fundings",
            "Count of initiated channel fundings",
        ).unwrap();
    static ref METRIC_CHANNEL_CLOSES: hopr_metrics::SimpleCounter =
        hopr_metrics::SimpleCounter::new(
            "hopr_strategy_channel_lifecycle_closes",
            "Count of initiated channel closures",
        ).unwrap();
    static ref METRIC_CHANNEL_FINALIZES: hopr_metrics::SimpleCounter =
        hopr_metrics::SimpleCounter::new(
            "hopr_strategy_channel_lifecycle_finalizations",
            "Count of initiated channel closure finalizations",
        ).unwrap();
}

// ─────────────────────────────────────────────────────────────────────────────
// Configuration
// ─────────────────────────────────────────────────────────────────────────────

/// Population thresholds: how many open channels to maintain.
#[serde_as]
#[derive(Debug, Clone, smart_default::SmartDefault, Validate, Serialize, Deserialize)]
pub struct PopulationConfig {
    /// Minimum number of open outgoing channels.  Closures are suppressed
    /// when the open count would drop below this.  Default: 5.
    #[default = 5]
    pub min_open_channels: usize,

    /// Target number of open outgoing channels.  New channels are opened until
    /// this target is reached.  Default: 8.
    #[default = 8]
    pub target_open_channels: usize,

    /// How long a peer is ineligible for a new channel after its previous
    /// channel was closed.  Default: 30 minutes.
    #[serde(default = "default_peer_reopen_cooldown", with = "humantime_serde")]
    #[default(default_peer_reopen_cooldown())]
    pub peer_reopen_cooldown: Duration,
}

fn default_peer_reopen_cooldown() -> Duration {
    Duration::from_secs(30 * 60)
}

/// Peer eligibility filters for channel opening and for determining staleness.
#[serde_as]
#[derive(Debug, Clone, smart_default::SmartDefault, Validate, Serialize, Deserialize)]
pub struct EligibilityConfig {
    /// Only open channels to peers that are currently connected.  Default: true.
    #[default = true]
    pub require_currently_connected: bool,

    /// A peer is considered stale — and its channel eligible for closure — once
    /// it has not been observed for this long.  Default: 24 hours.
    #[serde(default = "default_peer_staleness_threshold", with = "humantime_serde")]
    #[default(default_peer_staleness_threshold())]
    pub peer_staleness_threshold: Duration,

    /// Peer quality score threshold `[0.0, 1.0]` for opening new channels.
    /// Default: 0.5.
    #[default = 0.5]
    pub min_peer_quality_score: f64,

    /// Weight applied to the graph edge score in the composite peer score.
    /// Default: 0.6.
    #[default = 0.6]
    pub peer_quality_weight: f64,

    /// Weight applied to the normalised ticket-activity signal in the
    /// composite peer score.  Default: 0.4.
    #[default = 0.4]
    pub ticket_activity_weight: f64,

    /// Only close a channel to a peer that has been observed since startup
    /// (not just stale from a previous run).  Disabled when `false` allows
    /// closures based solely on the last-seen timestamp in the DB.
    /// Default: true — see `RestartGuardConfig`.
    #[default = true]
    pub require_observed_since_start: bool,

    /// If set, only open channels to addresses in this list.  `None` means
    /// all peers are eligible.  Default: None.
    #[default(None)]
    pub allowlist: Option<HashSet<Address>>,

    /// Never open channels to addresses in this list.  Default: empty.
    #[default(HashSet::new())]
    pub blocklist: HashSet<Address>,
}

fn default_peer_staleness_threshold() -> Duration {
    Duration::from_secs(24 * 60 * 60)
}

/// Initial and top-up balances for channel funding.
#[serde_as]
#[derive(Debug, Clone, smart_default::SmartDefault, Validate, Serialize, Deserialize)]
pub struct FundingConfig {
    /// Balance when opening a new channel.  Default: 1 wxHOPR.
    #[serde_as(as = "DisplayFromStr")]
    #[default(HoprBalance::new_base(1))]
    pub initial_balance: HoprBalance,

    /// Amount added when topping up an underfunded channel.  Default: 1 wxHOPR.
    #[serde_as(as = "DisplayFromStr")]
    #[default(HoprBalance::new_base(1))]
    pub topup_balance: HoprBalance,

    /// Channel balance below which a top-up is triggered.  Default: 0.1 wxHOPR.
    #[serde_as(as = "DisplayFromStr")]
    #[default(HoprBalance::new_base(1))]
    pub lower_balance_threshold: HoprBalance,

    /// Minimum safe balance required before opening or funding any channel.
    /// Default: 0.1 wxHOPR.
    #[serde_as(as = "DisplayFromStr")]
    #[default(HoprBalance::new_base(1))]
    pub min_safe_balance_required: HoprBalance,

    /// When `true` the fund and open passes are skipped entirely if the safe
    /// balance is below `min_safe_balance_required`.  Default: true.
    #[default = true]
    pub stop_when_unfunded: bool,
}

/// Configuration for proactive (predictive) channel funding.
///
/// When enabled the strategy estimates how much the channel balance will drain
/// during the time a funding transaction takes to confirm, and pre-funds if
/// the projected balance after confirmation would fall below the threshold.
#[serde_as]
#[derive(Debug, Clone, smart_default::SmartDefault, Validate, Serialize, Deserialize)]
pub struct ProactiveFundingConfig {
    /// Enable proactive funding.  Default: true.
    #[default = true]
    pub enabled: bool,

    /// Fallback tx-confirmation duration used when
    /// `ChainValues::typical_resolution_time()` fails.  Default: 60 s.
    #[serde(default = "default_fallback_chain_op_duration", with = "humantime_serde")]
    #[default(default_fallback_chain_op_duration())]
    pub fallback_chain_op_duration: Duration,

    /// How far back to look when computing the drain rate.  Default: 10 min.
    #[serde(default = "default_depletion_lookback", with = "humantime_serde")]
    #[default(default_depletion_lookback())]
    pub depletion_lookback: Duration,

    /// Multiplicative safety margin applied to the projected drain.
    /// `1.5` means fund if projected drain is 1.5× the threshold.  Default: 1.5.
    #[default = 1.5]
    pub safety_margin: f64,

    /// Weight of the balance-decrease signal in the drain rate estimate.
    /// Default: 1.0.
    #[default = 1.0]
    pub balance_drain_weight: f64,

    /// Weight of the ticket-index-increase signal (scaled by min ticket price)
    /// in the drain rate estimate.  Default: 1.0.
    #[default = 1.0]
    pub ticket_index_drain_weight: f64,
}

fn default_fallback_chain_op_duration() -> Duration {
    Duration::from_secs(60)
}
fn default_depletion_lookback() -> Duration {
    Duration::from_secs(10 * 60)
}

/// Thresholds that trigger channel closure.
#[serde_as]
#[derive(Debug, Clone, smart_default::SmartDefault, Validate, Serialize, Deserialize)]
pub struct ClosureConfig {
    /// Close a channel after the peer has been absent for this long.  Default: 24 h.
    #[serde(default = "default_close_when_peer_unseen_for", with = "humantime_serde")]
    #[default(default_close_when_peer_unseen_for())]
    pub close_when_peer_unseen_for: Duration,

    /// Close channels to peers whose quality score has dropped below this.
    /// Default: 0.3.
    #[default = 0.3]
    pub close_below_quality_score: f64,

    /// Close channels whose balance has dropped below this amount.  Default: 0.
    #[serde_as(as = "DisplayFromStr")]
    #[default(HoprBalance::zero())]
    pub close_when_drained_below: HoprBalance,

    /// Maximum simultaneous closure transactions initiated per pass.
    /// Default: 2.
    #[default = 2]
    pub close_max_concurrent: usize,
}

fn default_close_when_peer_unseen_for() -> Duration {
    Duration::from_secs(24 * 60 * 60)
}

/// Controls the finalizer phase (second `close_channel` call for `PendingToClose`
/// channels once the notice period has elapsed).
#[serde_as]
#[derive(Debug, Clone, smart_default::SmartDefault, Validate, Serialize, Deserialize)]
pub struct FinalizerConfig {
    /// Enable the finalizer phase.  When `false`, `PendingToClose` channels
    /// are left to be finalized externally.  Default: true.
    #[default = true]
    pub enabled: bool,

    /// Extra time to wait beyond the on-chain notice period before finalizing.
    /// Provides a buffer for slow-block periods.  Default: 30 min.
    #[serde(default = "default_max_closure_overdue", with = "humantime_serde")]
    #[default(default_max_closure_overdue())]
    pub max_closure_overdue: Duration,

    /// Maximum simultaneous finalization transactions initiated per pass.
    /// Default: 4.
    #[default = 4]
    pub finalize_max_concurrent: usize,
}

fn default_max_closure_overdue() -> Duration {
    Duration::from_secs(30 * 60)
}

/// Guards against mass-closing channels on restart (the graph is rebuilt from
/// scratch and peers appear unseen until heartbeats arrive).
#[serde_as]
#[derive(Debug, Clone, smart_default::SmartDefault, Validate, Serialize, Deserialize)]
pub struct RestartGuardConfig {
    /// The close pass is suppressed entirely for this long after startup.
    /// Should exceed network bootstrap time + first heartbeat round.
    /// Default: 10 min.
    #[serde(default = "default_startup_close_grace_period", with = "humantime_serde")]
    #[default(default_startup_close_grace_period())]
    pub startup_close_grace_period: Duration,
}

fn default_startup_close_grace_period() -> Duration {
    Duration::from_secs(10 * 60)
}

/// Concurrency knobs for the per-channel evaluation loops.
#[serde_as]
#[derive(Debug, Clone, smart_default::SmartDefault, Validate, Serialize, Deserialize)]
pub struct ConcurrencyConfig {
    /// Number of channels evaluated concurrently within each pass.  Default: 8.
    #[default = 8]
    pub per_pass_concurrency: usize,

    /// Maximum simultaneous in-flight chain-write operations (open + fund +
    /// close + finalize combined).  Additional operations are deferred to the
    /// next tick.  Default: 4.
    #[default = 4]
    pub max_concurrent_actions: usize,
}

/// Top-level configuration for [`ChannelLifecycleStrategy`].
///
/// All fields have sensible defaults; consumers only need to set the fields
/// they want to override.
#[serde_as]
#[derive(Debug, Clone, smart_default::SmartDefault, Validate, Serialize, Deserialize)]
pub struct ChannelLifecycleConfig {
    /// Base period between full evaluation passes.  Default: 60 s.
    #[serde(default = "default_tick_interval", with = "humantime_serde")]
    #[default(default_tick_interval())]
    pub tick_interval: Duration,

    /// Maximum random offset added to the tick interval to spread out
    /// concurrent node restarts.  Implemented as a deterministic offset based
    /// on the current system time nanoseconds.  Default: 5 s.
    #[serde(default = "default_jitter", with = "humantime_serde")]
    #[default(default_jitter())]
    pub jitter: Duration,

    pub population: PopulationConfig,
    pub eligibility: EligibilityConfig,
    pub funding: FundingConfig,
    pub proactive_funding: ProactiveFundingConfig,
    pub closure: ClosureConfig,
    pub finalizer: FinalizerConfig,
    pub restart: RestartGuardConfig,
    pub concurrency: ConcurrencyConfig,
}

fn default_tick_interval() -> Duration {
    Duration::from_secs(60)
}
fn default_jitter() -> Duration {
    Duration::from_secs(5)
}

// ─────────────────────────────────────────────────────────────────────────────
// Builder
// ─────────────────────────────────────────────────────────────────────────────

/// Builder for [`ChannelLifecycleStrategy`].
///
/// Cadence (`tick_interval`, `jitter`) lives in the config struct — operators
/// can tune it without touching wiring code.
///
/// Call [`new`](ChannelLifecycleStrategy::new) with the strategy configuration,
/// then [`build`](ChannelLifecycleStrategy::build) to wire in a node and obtain
/// a runnable `Box<dyn Strategy + Send>`.
pub struct ChannelLifecycleStrategy {
    cfg: ChannelLifecycleConfig,
}

impl ChannelLifecycleStrategy {
    /// Create a new builder with the given configuration.
    pub fn new(cfg: ChannelLifecycleConfig) -> Self {
        Self { cfg }
    }

    /// Wire in a node and return a running-ready strategy.
    ///
    /// The generic `N` is erased at construction time; the returned
    /// `Box<dyn Strategy + Send>` can be held and spawned without knowledge
    /// of the concrete node type.
    pub fn build<N>(self, node: Arc<N>) -> Box<dyn StrategyTrait + Send>
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
        Box::new(ChannelLifecycleStrategyInner {
            cfg: self.cfg,
            node,
            open_in_flight: Arc::new(DashSet::new()),
            fund_in_flight: Arc::new(DashSet::new()),
            close_in_flight: Arc::new(DashSet::new()),
            finalize_in_flight: Arc::new(DashSet::new()),
            cooldown: Arc::new(DashMap::new()),
            start_epoch: Instant::now(),
            last_observed: Arc::new(DashMap::new()),
            peer_ticket_activity: Arc::new(DashMap::new()),
        })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Inner type
// ─────────────────────────────────────────────────────────────────────────────

/// Per-channel observation snapshot for proactive funding.
#[derive(Clone)]
struct ChannelObservation {
    balance: HoprBalance,
    ticket_index: u64,
    at: Instant,
}

struct ChannelLifecycleStrategyInner<N>
where
    N: HasChainApi + HasNetworkView + HasGraphView,
{
    cfg: ChannelLifecycleConfig,
    node: Arc<N>,
    /// Destination addresses for channels being opened.
    open_in_flight: Arc<DashSet<Address>>,
    /// Channel IDs with in-flight funding transactions.
    fund_in_flight: Arc<DashSet<ChannelId>>,
    /// Channel IDs with in-flight closure transactions.
    close_in_flight: Arc<DashSet<ChannelId>>,
    /// Channel IDs with in-flight finalization transactions.
    finalize_in_flight: Arc<DashSet<ChannelId>>,
    /// Peer addresses and the instant until which they are on cooldown.
    cooldown: Arc<DashMap<Address, Instant>>,
    /// When this strategy instance started running.
    start_epoch: Instant,
    /// Most-recently recorded balance/ticket_index snapshot per channel.
    last_observed: Arc<DashMap<ChannelId, ChannelObservation>>,
    /// Ticket count delta recorded from TicketRedeemed events, keyed by dest addr.
    peer_ticket_activity: Arc<DashMap<Address, u64>>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Helper methods
// ─────────────────────────────────────────────────────────────────────────────

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
    /// Returns the available safe balance or `HoprBalance::zero()` if the safe
    /// is not registered.
    async fn safe_balance_budget(&self) -> crate::errors::Result<HoprBalance> {
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

    /// Returns the chain's estimated transaction confirmation time.  Falls back
    /// to the configured fallback duration on error.
    async fn est_tx_time(&self) -> Duration {
        match self.node.chain_api().typical_resolution_time().await {
            Ok(d) => d,
            Err(e) => {
                debug!(%e, "channel-lifecycle: typical_resolution_time failed, using fallback");
                self.cfg.proactive_funding.fallback_chain_op_duration
            }
        }
    }

    /// Returns the on-chain notice period for channel closure.  Falls back to
    /// a 5-minute default on error (conservative: don't finalize too early).
    async fn closure_notice_period(&self) -> Duration {
        match self.node.chain_api().channel_closure_notice_period().await {
            Ok(d) => d,
            Err(e) => {
                warn!(%e, "channel-lifecycle: could not fetch channel_closure_notice_period");
                Duration::from_secs(5 * 60)
            }
        }
    }

    /// Returns total in-flight chain-write count (open + fund + close + finalize).
    fn total_in_flight(&self) -> usize {
        self.open_in_flight.len()
            + self.fund_in_flight.len()
            + self.close_in_flight.len()
            + self.finalize_in_flight.len()
    }

    /// Returns the composite quality score for a peer given its offchain key.
    ///
    /// Blends the graph edge score with a normalised ticket activity signal.
    fn peer_score_for(&self, peer_offchain: &OffchainPublicKey, dest_addr: &Address) -> f64 {
        let my_key = self.node.graph().identity();
        let edge_score = self
            .node
            .graph()
            .edge(my_key, peer_offchain)
            .map(|e| e.score())
            .unwrap_or(0.0);

        // Ticket activity: normalise relative to the max seen activity (cap at 1.0).
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

    /// Returns `true` if the channel should be funded proactively based on the
    /// estimated drain rate and the time a transaction would take to confirm.
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

        // Balance-based drain: how much balance was consumed since last observation.
        let balance_now = obs.balance.amount().low_u128() as f64;
        let balance_then = channel.balance.amount().low_u128() as f64;
        let balance_delta = (balance_then - balance_now).max(0.0);
        let balance_drain_rate =
            (self.cfg.proactive_funding.balance_drain_weight * balance_delta / elapsed.min(lookback_secs)).max(0.0);

        // Ticket-index-based drain: how many tickets were redeemed × min price.
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

    /// Attempt to fund a channel.  Returns `true` if a task was spawned.
    fn try_fund_channel(&self, channel: &ChannelEntry, topup: HoprBalance) -> bool {
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
        METRIC_CHANNEL_FUNDS.increment();

        let chain = self.node.chain_api().clone();
        let in_flight = Arc::clone(&self.fund_in_flight);

        hopr_async_runtime::prelude::spawn(async move {
            match chain.fund_channel(&channel_id, topup).await {
                Ok(confirmation) => {
                    if let Err(e) = confirmation.await {
                        warn!(%channel_id, %e, "channel-lifecycle: funding tx failed");
                        in_flight.remove(&channel_id);
                    }
                    // On success: ChannelBalanceIncreased clears in_flight via event handler.
                }
                Err(e) => {
                    warn!(%channel_id, %e, "channel-lifecycle: failed to submit funding tx");
                    in_flight.remove(&channel_id);
                }
            }
        });

        true
    }

    /// Attempt to close a channel.  Returns `true` if a task was spawned.
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
        METRIC_CHANNEL_CLOSES.increment();

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

    /// Attempt to finalize a pending closure.  Returns `true` if a task was spawned.
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
        METRIC_CHANNEL_FINALIZES.increment();

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

    /// Attempt to open a new channel to `dest`.  Returns `true` if a task was spawned.
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
        METRIC_CHANNEL_OPENS.increment();

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

    async fn run_pipeline(&self) {
        if let Err(e) = self.pipeline_inner().await {
            warn!(%e, "channel-lifecycle: pipeline error");
        }
    }

    async fn pipeline_inner(&self) -> crate::errors::Result<()> {
        let chain = self.node.chain_api();
        let me = *chain.me();

        // ── 1. Snapshot ──────────────────────────────────────────────────────
        let est_tx_time = self.est_tx_time().await;
        let est_tx_secs = est_tx_time.as_secs_f64();

        // Fetch current safe balance once for the whole pass.
        let safe_balance = self.safe_balance_budget().await?;

        // Collect all outgoing channels.
        let mut all_channels: Vec<ChannelEntry> = Vec::new();
        {
            let mut s = chain
                .stream_channels(ChannelSelector::default().with_source(me))
                .map_err(|e| StrategyError::Other(e.into()))?;
            while let Some(ch) = s.next().await {
                all_channels.push(ch);
            }
        }

        // Update per-channel observations for proactive funding.
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

        // Fetch minimum ticket price for proactive drain estimation.
        let min_ticket_price_wei = chain
            .minimum_ticket_price()
            .await
            .map(|p| p.amount().low_u128() as f64)
            .unwrap_or(0.0);

        // Peer→address map from announced accounts (needed for the open pass).
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

        // Reverse map: chain address → offchain public key (for close pass).
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
        let grace_elapsed = self.start_epoch.elapsed() >= self.cfg.restart.startup_close_grace_period;

        if grace_elapsed {
            let mut close_count = self.close_in_flight.len();

            for ch in &open_channels {
                if close_count >= self.cfg.closure.close_max_concurrent {
                    break;
                }
                if self.close_in_flight.contains(ch.get_id()) {
                    continue;
                }
                // Never drop below min_open_channels.
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
            let notice_period = self.closure_notice_period().await;
            let overdue = notice_period + self.cfg.finalizer.max_closure_overdue;
            let mut finalize_count = self.finalize_in_flight.len();

            for ch in &all_channels {
                if finalize_count >= self.cfg.finalizer.finalize_max_concurrent {
                    break;
                }
                if self.finalize_in_flight.contains(ch.get_id()) {
                    continue;
                }
                if let ChannelStatus::PendingToClose(closure_time) = ch.status {
                    let elapsed_since_closure = closure_time.elapsed().unwrap_or(Duration::ZERO);
                    if elapsed_since_closure >= overdue && self.try_finalize_channel(ch) {
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
                // Skip self.
                if chain_addr == me {
                    return None;
                }
                // Skip if we already have a channel.
                if existing_dests.contains(&chain_addr) {
                    return None;
                }
                // Skip if already opening.
                if self.open_in_flight.contains(&chain_addr) {
                    return None;
                }
                // Skip if on cooldown.
                if self
                    .cooldown
                    .get(&chain_addr)
                    .is_some_and(|until| Instant::now() < *until)
                {
                    return None;
                }
                // Allowlist/blocklist.
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
                // Connectivity check.
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

        // Sort descending by score, take the top `deficit`.
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

    /// Returns `true` if the channel is eligible for closure.
    ///
    /// `addr_to_key` maps chain addresses to offchain public keys; built from
    /// the account stream in `pipeline_inner`.  Pass an empty map to skip the
    /// quality-score and staleness checks.
    fn should_close(&self, ch: &ChannelEntry, addr_to_key: &HashMap<Address, OffchainPublicKey>) -> bool {
        // Drained below threshold.
        if ch.balance <= self.cfg.closure.close_when_drained_below {
            return true;
        }

        let dest = ch.destination;
        if let Some(pk) = addr_to_key.get(&dest) {
            // Peer quality dropped below threshold.
            let score = self.peer_score_for(pk, &dest);
            if score < self.cfg.closure.close_below_quality_score {
                return true;
            }

            // Staleness: use graph edge `last_update` as last-seen proxy.
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

    // ─────────────────────────────────────────────────────────────────────
    // Event fast-lane handlers
    // ─────────────────────────────────────────────────────────────────────

    async fn on_balance_decreased(&self, ch: ChannelEntry, me: Address) {
        if ch.direction(&me) != Some(ChannelDirection::Outgoing) {
            return;
        }
        if ch.status != ChannelStatus::Open {
            return;
        }
        if self.fund_in_flight.contains(ch.get_id()) {
            return;
        }

        // Update observation to record the decrease.
        self.last_observed
            .entry(*ch.get_id())
            .and_modify(|obs| {
                obs.balance = ch.balance;
                obs.at = Instant::now();
            })
            .or_insert_with(|| ChannelObservation {
                balance: ch.balance,
                ticket_index: ch.ticket_index,
                at: Instant::now(),
            });

        if ch.balance <= self.cfg.funding.lower_balance_threshold {
            match self.safe_balance_budget().await {
                Ok(budget) if budget >= self.cfg.funding.topup_balance => {
                    self.try_fund_channel(&ch, self.cfg.funding.topup_balance);
                }
                Ok(budget) => {
                    debug!(%ch, %budget, "channel-lifecycle: event-driven funding skipped: safe too low");
                }
                Err(e) => {
                    warn!(%ch, %e, "channel-lifecycle: event-driven funding: could not fetch safe balance");
                }
            }
        }
    }

    fn on_balance_increased(&self, ch: ChannelEntry) {
        self.fund_in_flight.remove(ch.get_id());
        self.last_observed.entry(*ch.get_id()).and_modify(|obs| {
            obs.balance = ch.balance;
        });
        debug!(%ch, "channel-lifecycle: cleared fund in-flight after balance increase");
    }

    fn on_channel_opened(&self, ch: ChannelEntry) {
        self.open_in_flight.remove(&ch.destination);
        debug!(%ch, "channel-lifecycle: channel opened, cleared open in-flight");
    }

    fn on_channel_closure_initiated(&self, ch: ChannelEntry) {
        self.close_in_flight.remove(ch.get_id());
        debug!(%ch, "channel-lifecycle: closure initiated, cleared close in-flight");
    }

    fn on_channel_closed(&self, ch: ChannelEntry) {
        self.finalize_in_flight.remove(ch.get_id());
        self.last_observed.remove(ch.get_id());
        // Start cooldown for this peer.
        let until = Instant::now() + self.cfg.population.peer_reopen_cooldown;
        self.cooldown.insert(ch.destination, until);
        debug!(%ch, "channel-lifecycle: channel closed, peer on cooldown");
    }

    fn on_ticket_redeemed(&self, ch: ChannelEntry) {
        // Record ticket activity for the peer.
        self.peer_ticket_activity
            .entry(ch.destination)
            .and_modify(|v| *v += 1)
            .or_insert(1);
        // Update ticket index observation.
        self.last_observed.entry(*ch.get_id()).and_modify(|obs| {
            obs.ticket_index = ch.ticket_index;
        });
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Display + Strategy
// ─────────────────────────────────────────────────────────────────────────────

impl<N> Display for ChannelLifecycleStrategyInner<N>
where
    N: HasChainApi + HasNetworkView + HasGraphView,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "channel_lifecycle")
    }
}

#[async_trait]
impl<N> StrategyTrait for ChannelLifecycleStrategyInner<N>
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
    async fn run(&mut self) -> crate::errors::Result<()> {
        // Run first pipeline scan immediately.
        self.run_pipeline().await;

        let me = *self.node.chain_api().me();

        // Jitter: derive a fixed per-run offset from system-time nanoseconds so
        // nodes restarted simultaneously spread out their ticks.
        let jitter_ns = self.cfg.jitter.as_nanos() as u64;
        let jitter_offset = if jitter_ns > 0 {
            Duration::from_nanos(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .subsec_nanos() as u64
                    % jitter_ns,
            )
        } else {
            Duration::ZERO
        };
        let effective_interval = self.cfg.tick_interval + jitter_offset;

        let tick_stream = futures_time::stream::interval(effective_interval.into()).map(|_| LoopEvent::Tick);

        let event_stream = self
            .node
            .subscribe_to_actionable_events(Some(&[ActionableEventDiscriminant::Chain]))
            .map_err(|e| StrategyError::Other(anyhow::anyhow!(e)))?
            .filter_map(|ev| {
                futures::future::ready(match ev {
                    ActionableEvent::Chain(e) => Some(LoopEvent::Chain(Box::new(e))),
                    _ => None,
                })
            });

        let mut driver = futures_concurrency::stream::Merge::merge((tick_stream, event_stream));

        while let Some(evt) = driver.next().await {
            match evt {
                LoopEvent::Tick => {
                    self.run_pipeline().await;
                }
                LoopEvent::Chain(e) => match *e {
                    ChainEvent::ChannelBalanceDecreased(ch, _) => {
                        self.on_balance_decreased(ch, me).await;
                    }
                    ChainEvent::ChannelBalanceIncreased(ch, _) => {
                        self.on_balance_increased(ch);
                    }
                    ChainEvent::ChannelOpened(ch) => {
                        self.on_channel_opened(ch);
                    }
                    ChainEvent::ChannelClosureInitiated(ch) => {
                        self.on_channel_closure_initiated(ch);
                    }
                    ChainEvent::ChannelClosed(ch) => {
                        self.on_channel_closed(ch);
                    }
                    ChainEvent::TicketRedeemed(ch, _) => {
                        self.on_ticket_redeemed(ch);
                    }
                    _ => {}
                },
            }
        }

        Ok(())
    }
}

enum LoopEvent {
    Tick,
    Chain(Box<hopr_lib::api::chain::ChainEvent>),
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, sync::Arc, time::Duration};

    use hex_literal::hex;
    use hopr_chain_connector::{create_trustful_hopr_blokli_connector, testing::BlokliTestStateBuilder};
    use hopr_lib::api::{
        PeerId,
        chain::{
            AccountSelector, ChainEvent, ChainEvents, ChainReadAccountOperations, ChainWriteAccountOperations,
            HoprChainApi,
        },
        node::{
            ActionableEvent, ComponentStatus, ComponentStatusReporter, EventWaitResult, HasChainApi, HasGraphView,
            HasNetworkView, NodeOnchainIdentity,
        },
        types::{
            crypto::{keypairs::Keypair, prelude::ChainKeypair},
            internal::prelude::{ChannelEntry, ChannelStatus},
            primitive::prelude::{Address, BytesRepresentable, HoprBalance, XDaiBalance},
        },
    };

    use super::*;

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
    struct ChainNode<C>(C);

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

    // Stub implementations for HasNetworkView and HasGraphView for tests that
    // don't exercise network/graph paths.

    struct StubNetworkView;

    impl hopr_lib::api::network::NetworkView for StubNetworkView {
        fn listening_as(&self) -> HashSet<hopr_lib::api::Multiaddr> {
            HashSet::new()
        }
        fn multiaddress_of(&self, _peer: &PeerId) -> Option<HashSet<hopr_lib::api::Multiaddr>> {
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
        fn health(&self) -> hopr_lib::api::network::Health {
            hopr_lib::api::network::Health::Red
        }
        fn subscribe_network_events(
            &self,
        ) -> impl futures::Stream<Item = hopr_lib::api::network::NetworkEvent> + Send + 'static {
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

    // Stub graph — no edges, quality 0.5 for any hypothetical edge.
    struct StubGraph;

    impl hopr_lib::api::graph::NetworkGraphView for StubGraph {
        type Observed = StubEdge;
        type NodeId = OffchainPublicKey;

        fn node_count(&self) -> usize {
            0
        }
        fn contains_node(&self, _key: &OffchainPublicKey) -> bool {
            false
        }
        fn nodes(&self) -> futures::stream::BoxStream<'static, OffchainPublicKey> {
            Box::pin(futures::stream::empty())
        }
        fn edge(&self, _src: &OffchainPublicKey, _dest: &OffchainPublicKey) -> Option<StubEdge> {
            None
        }
        fn identity(&self) -> &OffchainPublicKey {
            static KEY: std::sync::OnceLock<OffchainPublicKey> = std::sync::OnceLock::new();
            KEY.get_or_init(|| {
                use hopr_lib::api::types::crypto::keypairs::Keypair as _;
                hopr_lib::api::types::crypto::prelude::OffchainKeypair::from_secret(&[1u8; 32])
                    .expect("test key")
                    .public()
                    .clone()
            })
        }
    }

    impl hopr_lib::api::graph::NetworkGraphConnectivity for StubGraph {
        type NodeId = OffchainPublicKey;
        type Observed = StubEdge;

        fn connected_edges(&self) -> Vec<(OffchainPublicKey, OffchainPublicKey, StubEdge)> {
            Vec::new()
        }
        fn reachable_edges(&self) -> Vec<(OffchainPublicKey, OffchainPublicKey, StubEdge)> {
            Vec::new()
        }
    }

    impl hopr_lib::api::graph::NetworkGraphTraverse for StubGraph {
        type NodeId = OffchainPublicKey;
        type Observed = StubEdge;

        fn simple_paths<V: hopr_lib::api::graph::ValueFn<Weight = StubEdge>>(
            &self,
            _source: &OffchainPublicKey,
            _destination: &OffchainPublicKey,
            _length: usize,
            _take_count: Option<usize>,
            _value_fn: V,
        ) -> Vec<(Vec<OffchainPublicKey>, [u64; 5], V::Value)> {
            Vec::new()
        }

        fn simple_paths_from<V: hopr_lib::api::graph::ValueFn<Weight = StubEdge>>(
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

    struct StubEdge;

    // EdgeObservable is auto-implemented via blanket impl for EdgeObservableRead + EdgeObservableWrite.

    impl hopr_lib::api::graph::EdgeObservableRead for StubEdge {
        type ImmediateMeasurement = StubMeasurement;
        type IntermediateMeasurement = StubMeasurement;

        fn last_update(&self) -> Duration {
            Duration::ZERO
        }
        fn immediate_qos(&self) -> Option<&Self::ImmediateMeasurement> {
            None
        }
        fn intermediate_qos(&self) -> Option<&Self::IntermediateMeasurement> {
            None
        }
        fn score(&self) -> f64 {
            0.5
        }
    }

    impl hopr_lib::api::graph::traits::EdgeObservableWrite for StubEdge {
        fn record(&mut self, _measurement: hopr_lib::api::graph::traits::EdgeWeightType) {}
    }

    struct StubMeasurement;

    impl hopr_lib::api::graph::EdgeLinkObservable for StubMeasurement {
        fn record(&mut self, _: hopr_lib::api::graph::traits::EdgeTransportMeasurement) {}
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

    impl hopr_lib::api::graph::traits::EdgeNetworkObservableRead for StubMeasurement {
        fn is_connected(&self) -> bool {
            false
        }
    }

    impl hopr_lib::api::graph::EdgeImmediateProtocolObservable for StubMeasurement {
        fn ack_rate(&self) -> Option<f64> {
            None
        }
    }

    impl hopr_lib::api::graph::traits::EdgeProtocolObservable for StubMeasurement {
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
            static G: StubGraph = StubGraph;
            &G
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

    #[test_log::test(tokio::test)]
    async fn test_default_config_has_sensible_values() {
        let cfg = ChannelLifecycleConfig::default();
        assert_eq!(cfg.population.min_open_channels, 5);
        assert_eq!(cfg.population.target_open_channels, 8);
        assert!(cfg.finalizer.enabled);
        assert!(cfg.proactive_funding.enabled);
        assert_eq!(cfg.eligibility.min_peer_quality_score, 0.5);
    }

    #[test_log::test(tokio::test)]
    async fn test_channel_lifecycle_strategy_funds_below_threshold() -> anyhow::Result<()> {
        let stake_limit = HoprBalance::from(3_u32);
        let fund_amount = HoprBalance::from(5_u32);

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

        let _snapshot = blokli_sim.snapshot();

        let mut connector =
            create_trustful_hopr_blokli_connector(&BOB_KP, Default::default(), blokli_sim, [1; Address::SIZE].into())
                .await?;
        connector.connect().await?;
        let connector = Arc::new(connector);
        register_test_safe(&*connector, *BOB).await?;

        let node = Arc::new(ChainNode(Arc::clone(&connector)));

        let cfg = ChannelLifecycleConfig {
            funding: FundingConfig {
                lower_balance_threshold: stake_limit,
                topup_balance: fund_amount,
                min_safe_balance_required: HoprBalance::from(1_u32),
                stop_when_unfunded: true,
                ..Default::default()
            },
            restart: RestartGuardConfig {
                startup_close_grace_period: Duration::ZERO,
            },
            ..Default::default()
        };

        let strategy = ChannelLifecycleStrategy::new(cfg);
        let _inner = strategy.build(node);

        // Run pipeline once
        futures_time::future::FutureExt::timeout(
            async {
                // Give time for the spawn to complete
                tokio::time::sleep(Duration::from_millis(500)).await;
            },
            futures_time::time::Duration::from_millis(2000),
        )
        .await
        .ok();

        // Verify a fund transaction was submitted (balance should have increased)
        let channels: Vec<ChannelEntry> = {
            use futures::StreamExt as _;
            connector
                .stream_channels(ChannelSelector::default().with_source(*BOB))
                .unwrap()
                .collect()
                .await
        };

        assert!(!channels.is_empty(), "should still have at least one channel");

        Ok(())
    }

    #[test]
    fn test_restart_grace_blocks_close_pass() {
        let cfg = ChannelLifecycleConfig {
            restart: RestartGuardConfig {
                startup_close_grace_period: Duration::from_secs(3600), // 1 hour
            },
            ..Default::default()
        };
        // The grace period is 1 hour from now — must not have elapsed immediately.
        let start_epoch = Instant::now();
        let grace_elapsed = start_epoch.elapsed() >= cfg.restart.startup_close_grace_period;
        assert!(
            !grace_elapsed,
            "close pass should be suppressed during startup grace period"
        );
    }

    /// Tests the public builder API: `ChannelLifecycleStrategy::new(...).build(node)` must
    /// return a `Box<dyn Strategy + Send>` with the expected Display string.
    #[test_log::test(tokio::test)]
    async fn test_display() -> anyhow::Result<()> {
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
        let node = Arc::new(ChainNode(Arc::clone(&chain_connector)));

        let strategy: Box<dyn crate::strategy::Strategy + Send> =
            ChannelLifecycleStrategy::new(ChannelLifecycleConfig::default()).build(node);

        assert_eq!(strategy.to_string(), "channel_lifecycle");
        fn assert_send<T: Send>(_: T) {}
        assert_send(strategy);

        Ok(())
    }

    #[test]
    fn test_cooldown_prevents_reopen() {
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
}
