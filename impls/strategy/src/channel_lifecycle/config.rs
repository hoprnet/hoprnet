use std::{collections::HashSet, time::Duration};

use hopr_api::types::primitive::prelude::{Address, HoprBalance};
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use validator::Validate;

/// Population thresholds: how many open channels to maintain.
#[serde_as]
#[derive(Debug, Clone, PartialEq, smart_default::SmartDefault, Validate, Serialize, Deserialize)]
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

#[inline]
fn default_peer_reopen_cooldown() -> Duration {
    Duration::from_secs(30 * 60)
}

/// Peer eligibility filters for channel opening and for determining staleness.
#[serde_as]
#[derive(Debug, Clone, PartialEq, smart_default::SmartDefault, Validate, Serialize, Deserialize)]
pub struct EligibilityConfig {
    /// Only open channels to peers that are currently connected.  Default: true.
    #[default = true]
    pub require_currently_connected: bool,

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

    /// Only close a channel when the peer has been observed since the strategy
    /// started running (i.e. `edge.last_update()` is more recent than
    /// `start_epoch.elapsed()`).  Protects against retiring channels for which
    /// the local view is still warming up after a restart.  Default: true.
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

/// Initial and top-up balances for channel funding.
#[serde_as]
#[derive(Debug, Clone, PartialEq, smart_default::SmartDefault, Validate, Serialize, Deserialize)]
pub struct FundingConfig {
    /// Balance when opening a new channel.  Default: 1 wxHOPR.
    #[serde_as(as = "DisplayFromStr")]
    #[default(HoprBalance::new_base(1))]
    pub initial_balance: HoprBalance,

    /// Amount added when topping up an underfunded channel.  Default: 1 wxHOPR.
    #[serde_as(as = "DisplayFromStr")]
    #[default(HoprBalance::new_base(1))]
    pub topup_balance: HoprBalance,

    /// Channel balance below which a top-up is triggered.  Default: 1 wxHOPR.
    #[serde_as(as = "DisplayFromStr")]
    #[default(HoprBalance::new_base(1))]
    pub lower_balance_threshold: HoprBalance,

    /// Minimum safe balance required before opening or funding any channel.
    /// Default: 1 wxHOPR.
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
#[derive(Debug, Clone, PartialEq, smart_default::SmartDefault, Validate, Serialize, Deserialize)]
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

#[inline]
fn default_fallback_chain_op_duration() -> Duration {
    Duration::from_secs(60)
}
#[inline]
fn default_depletion_lookback() -> Duration {
    Duration::from_secs(10 * 60)
}

/// Thresholds that trigger channel closure.
#[serde_as]
#[derive(Debug, Clone, PartialEq, smart_default::SmartDefault, Validate, Serialize, Deserialize)]
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

#[inline]
fn default_close_when_peer_unseen_for() -> Duration {
    Duration::from_secs(24 * 60 * 60)
}

/// Controls the finalizer phase (second `close_channel` call for `PendingToClose`
/// channels once the notice period has elapsed).
#[serde_as]
#[derive(Debug, Clone, PartialEq, smart_default::SmartDefault, Validate, Serialize, Deserialize)]
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

#[inline]
fn default_max_closure_overdue() -> Duration {
    Duration::from_secs(30 * 60)
}

/// Guards against mass-closing channels on restart (the graph is rebuilt from
/// scratch and peers appear unseen until heartbeats arrive).
#[serde_as]
#[derive(Debug, Clone, PartialEq, smart_default::SmartDefault, Validate, Serialize, Deserialize)]
pub struct RestartGuardConfig {
    /// The close pass is suppressed entirely for this long after startup.
    /// Should exceed network bootstrap time + first heartbeat round.
    /// Default: 10 min.
    #[serde(default = "default_startup_close_grace_period", with = "humantime_serde")]
    #[default(default_startup_close_grace_period())]
    pub startup_close_grace_period: Duration,
}

#[inline]
fn default_startup_close_grace_period() -> Duration {
    Duration::from_secs(10 * 60)
}

/// Concurrency knobs for the per-channel evaluation loops.
#[serde_as]
#[derive(Debug, Clone, PartialEq, smart_default::SmartDefault, Validate, Serialize, Deserialize)]
pub struct ConcurrencyConfig {
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
#[derive(Debug, Clone, PartialEq, smart_default::SmartDefault, Validate, Serialize, Deserialize)]
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

#[inline]
fn default_tick_interval() -> Duration {
    Duration::from_secs(60)
}
#[inline]
fn default_jitter() -> Duration {
    Duration::from_secs(5)
}
