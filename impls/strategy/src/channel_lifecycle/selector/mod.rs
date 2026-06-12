//! Pluggable peer-selection trait for the channel-lifecycle pipeline.
//!
//! [`Selector`] decouples the *selection policy* (which peers to open/close
//! channels with) from the pipeline invariants enforced around it (population
//! floor, `close_max_concurrent`, safe-balance budget, in-flight caps).
//!
//! The pipeline prepares a [`SelectorContext`] from the tick snapshot and
//! calls the two selector methods; hard invariants are enforced by the pipeline
//! after the selector returns its ranked lists.

use std::time::Duration;

use async_trait::async_trait;
use hopr_api::types::{
    crypto::prelude::OffchainPublicKey,
    internal::prelude::{ChannelEntry, ChannelId},
    primitive::prelude::Address,
};

use crate::channel_lifecycle::ChannelLifecycleConfig;

mod bucket;
mod default;
mod multi_objective;
mod stake;
mod subnet;

pub use bucket::{BucketCell, BucketView, LatencyBucket};
pub use default::DefaultSelector;
pub use multi_objective::MultiObjectiveSelector;
pub use stake::StakeView;
pub use subnet::SubnetBucket;

/// Signals that a [`Selector`] may request the pipeline to compute beyond the
/// per-tick defaults.  The pipeline skips expensive I/O (e.g. stake fetch) when
/// no active selector requests the corresponding signal.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SignalSet(u8);

impl SignalSet {
    /// Pipeline should fetch per-peer on-chain safe balance and populate `StakeView`.
    pub const STAKE: Self = Self(0b0001);

    pub const fn empty() -> Self {
        Self(0)
    }

    pub fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

impl std::ops::BitOr for SignalSet {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

/// Pre-computed graph-edge information for a single peer, derived from the
/// channel graph during the tick snapshot.  Passed into the selector so that
/// selector implementations do not need to hold references to the graph.
#[derive(Debug, Clone, Default)]
pub struct PeerEdgeInfo {
    /// Combined edge quality score: `probe_success_rate × latency_score(avg_rtt)`.
    /// `None` when no edge record exists in the graph.
    pub edge_score: Option<f64>,
    /// Age of the most recent graph observation for this edge.
    /// `Duration::ZERO` when no observations have been recorded.
    pub last_update: Duration,
    /// Raw round-trip time from graph observations.
    /// `None` when no edge record exists or latency has not been measured.
    pub average_latency: Option<Duration>,
    /// EMA probe success rate from the graph edge (messages received / sent).
    /// `0.0` when no edge record exists.
    pub probe_success_rate: f64,
    /// ACK rate from the immediate QoS measurement (`acks_received / messages_sent`).
    /// `None` when the graph edge has no immediate QoS sample.
    pub ack_rate: Option<f64>,
}

impl PeerEdgeInfo {
    /// `true` when at least one observation has been recorded for this edge.
    /// Used to gate close decisions that depend on graph data.
    pub fn has_probing_data(&self) -> bool {
        self.last_update > Duration::ZERO
    }
}

/// A candidate peer that has passed all hard eligibility filters and is
/// being offered to the selector for channel-open ranking.
///
/// Eligibility filters applied before a peer reaches this struct:
/// allowlist / blocklist / cooldown / open-in-flight / connectivity requirement.
#[derive(Debug, Clone)]
pub struct OpenCandidate {
    pub addr: Address,
    pub offchain_key: OffchainPublicKey,
    pub edge_info: PeerEdgeInfo,
    /// Normalised ticket-activity signal: `peer_ticket_count / max_activity` in `[0, 1]`.
    pub ticket_score: f64,
    /// Subnet bucket derived from the peer's multiaddresses.
    /// Used by the multi-objective selector for anonymity bucket placement.
    pub subnet: SubnetBucket,
}

/// An open channel being offered to the selector as a potential close target.
#[derive(Debug, Clone)]
pub struct CloseCandidate {
    pub channel: ChannelEntry,
    /// `None` when the peer's offchain key cannot be resolved from the address map.
    pub offchain_key: Option<OffchainPublicKey>,
    pub edge_info: PeerEdgeInfo,
    /// Normalised ticket-activity score, same scale as [`OpenCandidate::ticket_score`].
    pub ticket_score: f64,
}

/// Snapshot of all information the selector needs for one pipeline tick.
/// Prepared once in the snapshot phase and passed to both selector methods.
pub struct SelectorContext<'a> {
    pub cfg: &'a ChannelLifecycleConfig,
    /// How many new channels the open pass should fill.
    pub deficit: usize,
    /// Candidates pre-filtered by eligibility gates; selector ranks and returns a subset.
    pub open_candidates: &'a [OpenCandidate],
    /// All currently-open channels offered as potential close targets.
    pub close_candidates: &'a [CloseCandidate],
    /// How long this strategy instance has been running at the time of this tick.
    /// Used by the stale-peer guard to distinguish pre- and post-startup observations.
    pub start_epoch_elapsed: Duration,
    /// (latency, subnet) cells for all currently-open channels.
    /// Always populated; used by selectors that implement anonymity diversity logic.
    pub bucket_view: BucketView,
    /// Normalized on-chain safe-balance scores, keyed by peer chain address.
    /// Empty when the active selector did not request the `STAKE` signal.
    pub stake_view: StakeView,
}

/// Selects which peers to open channels with and which open channels to close.
///
/// # Contract
///
/// - Both methods receive a snapshot that is already pre-filtered by hard eligibility gates.  The selector *must not*
///   re-apply allowlist/blocklist logic; it only needs to rank and optionally filter further by its own policy.
/// - The pipeline enforces population floor, `close_max_concurrent`, and safe-balance budget *after* calling these
///   methods.
/// - Both methods are **async** to allow selectors that perform I/O (e.g. stake-balance fetches in future PRs).
#[async_trait]
pub trait Selector: Send + Sync {
    /// Signals that this selector requires the pipeline to compute beyond the defaults.
    /// The pipeline uses this to skip expensive operations (e.g. stake fetch) when they
    /// would not be consumed.
    fn required_signals(&self) -> SignalSet;

    /// Returns a ranked list of channels to close, ordered from highest
    /// close-priority to lowest.  The pipeline will close at most
    /// `close_max_concurrent` channels and will stop before the population
    /// drops below `min_open_channels`.
    async fn select_closes(&self, ctx: &SelectorContext<'_>) -> Vec<ChannelId>;

    /// Returns a ranked list of peers to open channels with, ordered from most
    /// to least preferred.  The pipeline will open at most `ctx.deficit`
    /// channels and will skip any for which the safe balance is insufficient.
    async fn select_opens(&self, ctx: &SelectorContext<'_>) -> Vec<(Address, OffchainPublicKey)>;
}
