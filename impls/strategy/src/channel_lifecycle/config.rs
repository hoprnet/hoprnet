use std::{collections::HashSet, time::Duration};

use bytesize::ByteSize;
use hopr_api::{
    chain::WinningProbability,
    types::primitive::prelude::{Address, HoprBalance, U256, UnitaryFloatOps},
};
use hopr_crypto_packet::prelude::HoprPacket;
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

/// Initial and top-up capacities for channel funding expressed as human-readable
/// data volumes.
///
/// The strategy converts each capacity to a wxHOPR amount at runtime using the
/// live on-chain ticket price and winning probability via [`FundingConfig::resolve`].
///
/// **Conversion formula** (RFC-0005 §3.2):
/// ```text
/// packets     = ceil(capacity_bytes / HoprPacket::PAYLOAD_SIZE)
/// funding_wei = ticket_price_wei × packets × assumed_hops / win_prob
/// ```
/// `assumed_hops` is the number of paid downstream relay hops.  Defaulting to 3
/// (the protocol maximum) ensures the channel is never under-funded when paths
/// use the full relay depth.
#[derive(Debug, Clone, PartialEq, smart_default::SmartDefault, Validate, Serialize, Deserialize)]
pub struct FundingConfig {
    /// Data volume a newly opened channel's stake should be able to carry.
    /// Default: 1 GiB.
    #[default(ByteSize::gib(1))]
    pub initial_capacity: ByteSize,

    /// Data volume added to a channel's stake when it is topped up.
    /// Default: 1 GiB.
    #[default(ByteSize::mib(512))]
    pub topup_capacity: ByteSize,

    /// The channel balance (expressed as data capacity) below which a top-up is
    /// triggered.  Default: 128 MiB.
    #[default(ByteSize::mib(512))]
    pub lower_capacity_threshold: ByteSize,

    /// Minimum safe balance (expressed as data capacity) required before the
    /// strategy opens or funds any channel.  Default: 1 GiB.
    #[default(ByteSize::mib(512))]
    pub min_safe_capacity_required: ByteSize,

    /// When `true` the fund and open passes are skipped entirely if the safe
    /// balance is below `min_safe_capacity_required`.  Default: true.
    #[default = true]
    pub stop_when_unfunded: bool,

    /// Number of paid downstream relay hops assumed when sizing the channel
    /// stake.  Must be ≥ 1 and ≤ [`RoutingOptions::MAX_INTERMEDIATE_HOPS`][routing] (3).
    /// Default: 3.
    ///
    /// [routing]: hopr_api::types::internal::routing::RoutingOptions
    #[default = 3]
    #[validate(range(min = 1, max = 3))]
    pub assumed_hops: u32,
}

// ─────────────────────────────────────────────────────────────────────────────
// Capacity → wxHOPR conversion
// ─────────────────────────────────────────────────────────────────────────────

/// wxHOPR amounts resolved from [`FundingConfig`] at the current ticket
/// economics.  Computed once per pipeline tick and threaded through the fund,
/// open, and close-decision paths.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ResolvedFunding {
    /// Initial balance when opening a new channel.
    pub initial_balance: HoprBalance,
    /// Amount added when topping up an underfunded channel.
    pub topup_balance: HoprBalance,
    /// Channel balance below which a top-up is triggered.
    pub lower_balance_threshold: HoprBalance,
    /// Minimum safe balance required before opening or funding any channel.
    pub min_safe_balance_required: HoprBalance,
}

/// Convert a data `capacity` to a wxHOPR balance using the live ticket economics.
///
/// The formula matches the ticket-issuance math in `HoprTicketFactory`:
/// ```text
/// packets     = ceil(capacity_bytes / HoprPacket::PAYLOAD_SIZE)
/// funding_wei = ticket_price_wei × packets × hops / win_prob
/// ```
/// Returns [`HoprBalance::zero`] for zero capacity.
/// Falls back to a `win_prob`-independent estimate (`price × packets × hops`)
/// when `win_prob` is zero or converting it to f64 would produce a non-positive
/// value, to avoid dividing by zero.
pub(crate) fn capacity_to_balance(
    capacity: ByteSize,
    price: HoprBalance,
    win_prob: WinningProbability,
    hops: u32,
) -> HoprBalance {
    let bytes = capacity.as_u64();
    if bytes == 0 {
        return HoprBalance::zero();
    }

    // ceil(bytes / PAYLOAD_SIZE)
    let payload = HoprPacket::PAYLOAD_SIZE as u64;
    let packets = bytes.div_ceil(payload);

    // ticket_price_wei × packets × hops — saturating; overflow becomes U256::MAX
    let wei_base = price
        .amount()
        .saturating_mul(U256::from(packets))
        .saturating_mul(U256::from(hops));

    // Divide by win_prob (≤ 1.0); fall back to no division if prob is degenerate.
    let wp: f64 = win_prob.into();
    let wei = if wp > 0.0 {
        match wei_base.div_f64(wp) {
            Ok(v) => v,
            Err(_) => wei_base, // degenerate: return undiscounted value
        }
    } else {
        wei_base
    };

    HoprBalance::from(wei)
}

impl FundingConfig {
    /// Resolve all data-capacity fields to wxHOPR amounts at the given ticket
    /// economics.  Called once per pipeline tick.
    pub(crate) fn resolve(&self, price: HoprBalance, win_prob: WinningProbability) -> ResolvedFunding {
        let hops = self.assumed_hops;
        ResolvedFunding {
            initial_balance: capacity_to_balance(self.initial_capacity, price, win_prob, hops),
            topup_balance: capacity_to_balance(self.topup_capacity, price, win_prob, hops),
            lower_balance_threshold: capacity_to_balance(self.lower_capacity_threshold, price, win_prob, hops),
            min_safe_balance_required: capacity_to_balance(self.min_safe_capacity_required, price, win_prob, hops),
        }
    }
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

/// Per-axis weights for the multi-objective channel selector.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectorWeights {
    /// Weight of the latency axis.
    pub latency: f64,
    /// Weight of the combined trust axis (probe success + ACK rate + ticket activity).
    pub trust: f64,
    /// Weight of the on-chain stake axis.
    pub stake: f64,
    /// Weight of the anonymity (bucket diversity) axis.
    pub anonymity: f64,
    /// Inner weight for probe success rate within the trust axis.  Default: 0.50.
    pub trust_probe: f64,
    /// Inner weight for ACK rate within the trust axis.  Default: 0.35.
    pub trust_ack: f64,
    /// Inner weight for ticket activity within the trust axis.  Default: 0.15.
    pub trust_ticket: f64,
}

impl SelectorWeights {
    pub const fn new(latency: f64, trust: f64, stake: f64, anonymity: f64) -> Self {
        Self {
            latency,
            trust,
            stake,
            anonymity,
            trust_probe: 0.50,
            trust_ack: 0.35,
            trust_ticket: 0.15,
        }
    }
}

/// Configuration for the multi-objective channel selector.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultiObjectiveSelectorConfig {
    pub weights: SelectorWeights,
    /// Maximum number of opens initiated per strategy tick.  Selector returns at most this many
    /// candidates; the pipeline may dispatch fewer due to safe-balance or concurrency limits.
    pub open_per_tick: usize,
    /// Maximum number of closes initiated per strategy tick.
    pub close_per_tick: usize,
    /// Minimum number of distinct `(latency, subnet)` cells that must be populated among open
    /// channels.  The open pass fills underrepresented cells first; the close pass vetoes closing
    /// the sole occupant of any cell.  `Unknown` subnet peers are excluded from the floor.
    pub k_floor: usize,
    /// Hysteresis gap between the open quality threshold
    /// (`eligibility.min_peer_quality_score`) and the effective close quality
    /// threshold.  The close threshold used by this selector is
    /// `max(0, min_peer_quality_score − hysteresis_gap)`, which is
    /// typically lower than `closure.close_below_quality_score`.  A wider gap
    /// suppresses churn — once open, a channel stays open until quality is
    /// substantially worse than the open bar.
    pub hysteresis_gap: f64,
}

impl MultiObjectiveSelectorConfig {
    pub fn low_latency() -> Self {
        Self {
            weights: SelectorWeights::new(0.70, 0.20, 0.05, 0.05),
            open_per_tick: 4,
            close_per_tick: 4,
            k_floor: 2,
            hysteresis_gap: 0.10,
        }
    }

    pub fn balanced() -> Self {
        Self {
            weights: SelectorWeights::new(0.35, 0.30, 0.15, 0.20),
            open_per_tick: 2,
            close_per_tick: 2,
            k_floor: 3,
            hysteresis_gap: 0.20,
        }
    }

    pub fn dispersed() -> Self {
        Self {
            weights: SelectorWeights::new(0.20, 0.20, 0.10, 0.50),
            open_per_tick: 2,
            close_per_tick: 2,
            k_floor: 4,
            hysteresis_gap: 0.20,
        }
    }

    pub fn economical() -> Self {
        Self {
            weights: SelectorWeights::new(0.30, 0.30, 0.30, 0.10),
            open_per_tick: 1,
            close_per_tick: 1,
            k_floor: 2,
            hysteresis_gap: 0.40,
        }
    }

    /// Returns an error message if the inner trust weights do not approximately sum to 1.0.
    /// Intended for use by `Custom` profile validation.
    pub fn validate_trust_weights(&self) -> Result<(), String> {
        let sum = self.weights.trust_probe + self.weights.trust_ack + self.weights.trust_ticket;
        if (sum - 1.0).abs() > 0.01 {
            Err(format!(
                "trust inner weights must sum to ~1.0 (got {:.4}): probe={}, ack={}, ticket={}",
                sum, self.weights.trust_probe, self.weights.trust_ack, self.weights.trust_ticket
            ))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod config_tests {
    use anyhow::Context as _;

    use super::*;

    #[test]
    fn all_named_profiles_have_valid_trust_weights() {
        for cfg in [
            MultiObjectiveSelectorConfig::low_latency(),
            MultiObjectiveSelectorConfig::balanced(),
            MultiObjectiveSelectorConfig::dispersed(),
            MultiObjectiveSelectorConfig::economical(),
        ] {
            assert!(
                cfg.validate_trust_weights().is_ok(),
                "profile has invalid trust weights: {:?}",
                cfg.validate_trust_weights()
            );
        }
    }

    #[test]
    fn invalid_trust_weights_are_caught() {
        let mut cfg = MultiObjectiveSelectorConfig::low_latency();
        cfg.weights.trust_probe = 0.9;
        cfg.weights.trust_ack = 0.9;
        cfg.weights.trust_ticket = 0.9; // sum = 2.7
        assert!(cfg.validate_trust_weights().is_err());
    }

    // ── capacity_to_balance unit tests ──────────────────────────────────────

    /// Helper: build a `HoprBalance` from raw wei as u128.
    fn balance_from_wei(wei: u128) -> HoprBalance {
        HoprBalance::from(U256::from(wei))
    }

    /// Helper: HOPR ticket price of 0.01 wxHOPR expressed in wei.
    /// 0.01 × 10^18 = 10_000_000_000_000_000
    const PRICE_WEI: u128 = 10_000_000_000_000_000;

    #[test]
    fn zero_capacity_returns_zero() -> anyhow::Result<()> {
        let price = balance_from_wei(PRICE_WEI);
        let wp = WinningProbability::try_from(1.0f64).context("create win_prob")?;
        assert_eq!(capacity_to_balance(ByteSize::b(0), price, wp, 3), HoprBalance::zero());
        Ok(())
    }

    #[test]
    fn exact_packet_count_win_prob_one() -> anyhow::Result<()> {
        // capacity = 10 × PAYLOAD_SIZE → exactly 10 packets
        let price = balance_from_wei(PRICE_WEI);
        let wp = WinningProbability::try_from(1.0f64).context("create win_prob")?;
        let capacity = ByteSize::b((HoprPacket::PAYLOAD_SIZE * 10) as u64);
        let result = capacity_to_balance(capacity, price, wp, 3);
        // expected: PRICE_WEI * 10 packets * 3 hops / 1.0
        let expected = balance_from_wei(PRICE_WEI * 10 * 3);
        assert_eq!(result, expected, "exact 10 packets, win_prob=1.0");
        Ok(())
    }

    #[test]
    fn half_win_prob_doubles_funding() -> anyhow::Result<()> {
        // win_prob = 0.5 → face-value doubles vs. win_prob = 1.0
        let price = balance_from_wei(PRICE_WEI);
        let wp_full = WinningProbability::try_from(1.0f64).context("create wp_full")?;
        let wp_half = WinningProbability::try_from(0.5f64).context("create wp_half")?;
        let capacity = ByteSize::b((HoprPacket::PAYLOAD_SIZE * 10) as u64);
        let full = capacity_to_balance(capacity, price, wp_full, 3);
        let half = capacity_to_balance(capacity, price, wp_half, 3);
        // half should be approximately double full
        let ratio = half.amount().low_u128() as f64 / full.amount().low_u128() as f64;
        assert!((ratio - 2.0).abs() < 0.01, "ratio={ratio}");
        Ok(())
    }

    #[test]
    fn sub_packet_capacity_rounds_up_to_one_packet() -> anyhow::Result<()> {
        // 1 byte → ceil(1 / PAYLOAD_SIZE) = 1 packet
        let price = balance_from_wei(PRICE_WEI);
        let wp = WinningProbability::try_from(1.0f64).context("create win_prob")?;
        let result = capacity_to_balance(ByteSize::b(1), price, wp, 1);
        let expected = balance_from_wei(PRICE_WEI * 1 * 1);
        assert_eq!(result, expected, "1 byte rounds up to 1 packet");
        Ok(())
    }

    #[test]
    fn assumed_hops_scales_linearly() -> anyhow::Result<()> {
        let price = balance_from_wei(PRICE_WEI);
        let wp = WinningProbability::try_from(1.0f64).context("create win_prob")?;
        let capacity = ByteSize::b(HoprPacket::PAYLOAD_SIZE as u64);
        let h1 = capacity_to_balance(capacity, price, wp, 1);
        let h3 = capacity_to_balance(capacity, price, wp, 3);
        assert_eq!(
            h3.amount(),
            h1.amount().saturating_mul(U256::from(3u64)),
            "3 hops should be 3× 1 hop"
        );
        Ok(())
    }

    // ── FundingConfig::resolve ───────────────────────────────────────────────

    #[test]
    fn resolve_maps_all_four_fields() -> anyhow::Result<()> {
        let cfg = FundingConfig::default();
        let price = balance_from_wei(PRICE_WEI);
        let wp = WinningProbability::try_from(1.0f64).context("create win_prob")?;
        let resolved = cfg.resolve(price, wp);

        // Each resolved balance must match what capacity_to_balance produces independently.
        assert_eq!(
            resolved.initial_balance,
            capacity_to_balance(cfg.initial_capacity, price, wp, cfg.assumed_hops)
        );
        assert_eq!(
            resolved.topup_balance,
            capacity_to_balance(cfg.topup_capacity, price, wp, cfg.assumed_hops)
        );
        assert_eq!(
            resolved.lower_balance_threshold,
            capacity_to_balance(cfg.lower_capacity_threshold, price, wp, cfg.assumed_hops)
        );
        assert_eq!(
            resolved.min_safe_balance_required,
            capacity_to_balance(cfg.min_safe_capacity_required, price, wp, cfg.assumed_hops)
        );
        Ok(())
    }

    // ── FundingConfig validation & defaults ─────────────────────────────────

    #[test]
    fn default_config_passes_validation() {
        use validator::Validate as _;
        let cfg = FundingConfig::default();
        assert!(cfg.validate().is_ok(), "default FundingConfig should be valid");
    }

    #[test]
    fn assumed_hops_zero_is_rejected() {
        use validator::Validate as _;
        let mut cfg = FundingConfig::default();
        cfg.assumed_hops = 0;
        assert!(cfg.validate().is_err(), "assumed_hops = 0 must be rejected");
    }

    #[test]
    fn default_assumed_hops_is_three() {
        assert_eq!(FundingConfig::default().assumed_hops, 3);
    }

    // ── Serde round-trip ─────────────────────────────────────────────────────

    #[test]
    fn funding_config_serde_roundtrip() -> anyhow::Result<()> {
        // ByteSize serializes to human-readable strings ("5 GiB", "512 MiB", etc.).
        // Use exact IEC multiples so serialize → deserialize is lossless.
        let cfg = FundingConfig {
            initial_capacity: ByteSize::gib(5),
            topup_capacity: ByteSize::mib(512),
            lower_capacity_threshold: ByteSize::mib(128),
            min_safe_capacity_required: ByteSize::gib(2),
            stop_when_unfunded: false,
            assumed_hops: 2,
        };
        let json = serde_json::to_string(&cfg).context("serialize")?;
        let back: FundingConfig = serde_json::from_str(&json).context("deserialize")?;
        assert_eq!(cfg, back);
        Ok(())
    }
}

/// Selector profile selection for [`ChannelLifecycleConfig`].
///
/// Defaults to `Default` (existing `DefaultSelector` behavior, zero behavior change).
/// Operators opt in to multi-objective selection by choosing a named profile or `Custom`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SelectorProfile {
    /// Original weighted-sum selector.  Zero behavior change from pre-redesign deployments.
    #[default]
    Default,
    LowLatency,
    Balanced,
    Dispersed,
    Economical,
    Custom(MultiObjectiveSelectorConfig),
}

impl SelectorProfile {
    /// Returns the `MultiObjectiveSelectorConfig` for this profile, or `None` for `Default`.
    pub fn multi_objective_config(&self) -> Option<MultiObjectiveSelectorConfig> {
        match self {
            Self::Default => None,
            Self::LowLatency => Some(MultiObjectiveSelectorConfig::low_latency()),
            Self::Balanced => Some(MultiObjectiveSelectorConfig::balanced()),
            Self::Dispersed => Some(MultiObjectiveSelectorConfig::dispersed()),
            Self::Economical => Some(MultiObjectiveSelectorConfig::economical()),
            Self::Custom(cfg) => Some(cfg.clone()),
        }
    }
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
    /// Open/close selection policy.  Defaults to the original weighted-sum selector.
    #[default(SelectorProfile::Default)]
    pub selector: SelectorProfile,
}

#[inline]
fn default_tick_interval() -> Duration {
    Duration::from_secs(60)
}
#[inline]
fn default_jitter() -> Duration {
    Duration::from_secs(5)
}
