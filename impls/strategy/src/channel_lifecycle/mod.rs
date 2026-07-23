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
//!                                                       │ deadline + max_closure_overdue
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

mod config;
pub use config::*;

mod events;
mod pipeline;
pub mod selector;
mod strategy;
use std::{collections::HashMap, sync::Arc, time::Instant};

use dashmap::{DashMap, DashSet};
use hopr_api::{
    PeerId,
    types::{
        crypto::prelude::OffchainPublicKey,
        internal::prelude::ChannelId,
        primitive::prelude::{Address, HoprBalance},
    },
};
use parking_lot::Mutex;
use selector::Selector;
pub use strategy::ChannelLifecycleStrategy;

#[cfg(all(feature = "telemetry", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_CHANNEL_OPENS: hopr_api::types::telemetry::SimpleCounter =
        hopr_api::types::telemetry::SimpleCounter::new(
            "hopr_strategy_channel_lifecycle_opens",
            "Count of initiated channel opens",
        ).unwrap();
    static ref METRIC_CHANNEL_FUNDS: hopr_api::types::telemetry::SimpleCounter =
        hopr_api::types::telemetry::SimpleCounter::new(
            "hopr_strategy_channel_lifecycle_fundings",
            "Count of initiated channel fundings",
        ).unwrap();
    static ref METRIC_CHANNEL_CLOSES: hopr_api::types::telemetry::SimpleCounter =
        hopr_api::types::telemetry::SimpleCounter::new(
            "hopr_strategy_channel_lifecycle_closes",
            "Count of initiated channel closures",
        ).unwrap();
    static ref METRIC_CHANNEL_FINALIZES: hopr_api::types::telemetry::SimpleCounter =
        hopr_api::types::telemetry::SimpleCounter::new(
            "hopr_strategy_channel_lifecycle_finalizations",
            "Count of initiated channel closure finalizations",
        ).unwrap();

    // ── Diversity / anonymity ─────────────────────────────────────────────────
    /// Shannon-entropy-based effective number of distinct (latency, subnet) cells.
    static ref METRIC_EFFECTIVE_BUCKETS: hopr_api::types::telemetry::SimpleGauge =
        hopr_api::types::telemetry::SimpleGauge::new(
            "hopr_strategy_channel_lifecycle_effective_buckets",
            "Effective number of distinct (latency, subnet) bucket cells among open channels (2^H)",
        ).unwrap();

    /// Per-cell channel count, labelled by the cell description.
    static ref METRIC_BUCKET_COUNT: hopr_api::types::telemetry::MultiGauge =
        hopr_api::types::telemetry::MultiGauge::new(
            "hopr_strategy_channel_lifecycle_bucket_count",
            "Number of open channels in each (latency, subnet) bucket cell",
            &["cell"],
        ).unwrap();

    /// Variance of round-trip times across all open channels, in milliseconds.
    static ref METRIC_LATENCY_VARIANCE_MS: hopr_api::types::telemetry::SimpleGauge =
        hopr_api::types::telemetry::SimpleGauge::new(
            "hopr_strategy_channel_lifecycle_latency_variance_ms",
            "Variance of round-trip times (ms) across all open channels",
        ).unwrap();

    /// Number of distinct /24 or /48 subnet prefixes among open channels.
    static ref METRIC_SUBNET_COUNT: hopr_api::types::telemetry::SimpleGauge =
        hopr_api::types::telemetry::SimpleGauge::new(
            "hopr_strategy_channel_lifecycle_subnet_count",
            "Number of distinct subnet prefixes among open channels",
        ).unwrap();

    /// Average per-axis score across all open-channel candidates for the last tick.
    /// Only non-zero when the multi-objective selector is active.
    static ref METRIC_SCORE_AXIS: hopr_api::types::telemetry::MultiGauge =
        hopr_api::types::telemetry::MultiGauge::new(
            "hopr_strategy_channel_lifecycle_score_axis",
            "Average per-axis score across open candidates in the last strategy tick",
            &["axis"],
        ).unwrap();
}

/// Per-channel observation snapshot used by the proactive funding estimate.
#[derive(Clone)]
struct ChannelObservation {
    balance: HoprBalance,
    ticket_index: u64,
    at: Instant,
}

/// Cached `peer_id → (offchain key, chain address)` map plus the timestamp at
/// which it was last refreshed.  Lets the snapshot pass skip the full account
/// stream on most ticks.
struct PeerAddrCache {
    refreshed_at: Instant,
    map: HashMap<PeerId, (OffchainPublicKey, Address)>,
}

/// The running strategy instance.  Generic over the node type `N` so that
/// callers can provide any node implementation satisfying the required traits.
///
/// Constructed via [`ChannelLifecycleStrategy::build`]; the builder erases `N`
/// behind `Box<dyn Strategy + Send>`.
struct ChannelLifecycleStrategyInner<N> {
    cfg: ChannelLifecycleConfig,
    node: Arc<N>,
    /// Pluggable selection policy.  Decides which peers to open channels with
    /// and which open channels to retire.  Pipeline invariants (population
    /// floor, concurrent-action caps, safe-balance budget) are enforced by the
    /// pipeline regardless of the selector's choices.
    selector: Arc<dyn Selector>,
    /// Destination addresses for channels currently being opened.
    open_in_flight: Arc<DashSet<Address>>,
    /// Channel IDs with an in-flight funding transaction.
    fund_in_flight: Arc<DashSet<ChannelId>>,
    /// Channel IDs with an in-flight closure transaction.
    close_in_flight: Arc<DashSet<ChannelId>>,
    /// Channel IDs with an in-flight finalization transaction.
    finalize_in_flight: Arc<DashSet<ChannelId>>,
    /// Peer addresses mapped to the `Instant` when their cooldown expires.
    cooldown: Arc<DashMap<Address, Instant>>,
    /// When this strategy instance started; used by the restart guard.
    start_epoch: Instant,
    /// Most-recently recorded balance/ticket_index snapshot per channel.
    last_observed: Arc<DashMap<ChannelId, ChannelObservation>>,
    /// Cumulative ticket count increments from `TicketRedeemed` events.
    peer_ticket_activity: Arc<DashMap<Address, u64>>,
    /// TTL-cached peer-id → (offchain key, chain address) map.  Avoids
    /// streaming the full on-chain account list on every tick.
    peer_addr_cache: Arc<Mutex<Option<PeerAddrCache>>>,
    /// Economics resolved by the most-recent pipeline tick, shared with the
    /// event-driven funding handler so it reuses per-tick values instead of
    /// issuing fresh chain RPC calls on every balance-decrease event.
    last_resolved_funding: Arc<Mutex<Option<ResolvedFunding>>>,
}
