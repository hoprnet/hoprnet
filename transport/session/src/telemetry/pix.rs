//! Node-level PIX Exit instruments.
//!
//! The bookkeeping that drives these lives in [`crate::supervision::telemetry`] and is compiled
//! unconditionally; this module is only the emission boundary. Everything here is a thin
//! `pub(crate) fn` over a `lazy_static!` instrument, taking a closed enum rather than a string so
//! that no call site can invent a label value.
//!
//! # Prefix
//!
//! These use `hopr_pix_*` rather than the `hopr_session_pix_*` of their per-Session siblings, and
//! the difference is not cosmetic. `hopr-types`' `meter_for_metric` routes an instrument to a meter
//! provider by *name prefix*, and `hopr_session_*` is routed to the OTLP-only provider — see
//! `OTLP.md`, which records that those are not exposed on the Prometheus `/metrics` endpoint. These
//! aggregates are the ones an operator dashboards and alerts on, so they belong on `/metrics`,
//! alongside the existing `hopr_pix_share_insert_failures`.
//!
//! The per-Session gauges keep their spelling and their OTLP route. They answer a different
//! question for a different consumer.
//!
//! # Metric contract
//!
//! | Metric | Type | Unit | Updated at | Labels |
//! | --- | --- | --- | --- | --- |
//! | `hopr_pix_sessions_active` | MultiGauge | Sessions | supervisor turn (census delta) | `gate_mode` = `predeposit\|funded` |
//! | `hopr_pix_cycles_active` | MultiGauge | cycles | supervisor turn (census delta) | `phase` = `awaiting_commitment\|awaiting_deposit\|recovering\|paid_tail` |
//! | `hopr_pix_predeposit_exposure_packets` | SimpleGauge | packets | supervisor turn (census delta) | — |
//! | `hopr_pix_live_cycle_bytes` | SimpleGauge | bytes | admission reservation / release | — |
//! | `hopr_pix_cycle_bytes_reserved_total` | SimpleCounter | bytes | admission reservation | — |
//! | `hopr_pix_cycle_bytes_released_total` | SimpleCounter | bytes | reservation release | — |
//! | `hopr_pix_cycles_total` | MultiCounter | cycles | supervisor transition (event-counted) | `event` = `requested\|committed\|funded\|recovered\|failed` |
//! | `hopr_pix_admission_rejections_total` | MultiCounter | refusals | incoming Start path | `reason` — see [`PixAdmissionRejection`] |
//! | `hopr_pix_egress_packets_total` | MultiCounter | packets | supervisor turn (delta-counted from the gate) | `mode` = `predeposit\|funded` |
//! | `hopr_pix_gate_blocks_total` | MultiCounter | episodes | first refusal of an episode | `reason` = `predeposit_exhausted\|share_lag\|closed` |
//! | `hopr_pix_gate_block_seconds` | MultiHistogram | seconds | episode end | `reason` = `predeposit_exhausted\|share_lag` |
//!
//! The live-set gauges are **delta-counted** from a recomputed census and the counters are
//! **event-counted** at the transition that changes the source-of-truth state. Neither is derived
//! from a heap estimate or inferred from logs.
//!
//! # Operator queries
//!
//! *Funded versus unfunded Sessions.* A node whose Sessions are mostly unfunded is one whose Entries
//! are not depositing:
//!
//! ```promql
//! sum by (gate_mode) (hopr_pix_sessions_active)
//! ```
//!
//! *Rising predeposit exposure.* Service given away and not yet converted into paid service. It is
//! bounded per Session by `max_predeposit_packets`, so a sustained climb means the *number* of
//! unfunded fronts is climbing:
//!
//! ```promql
//! hopr_pix_predeposit_exposure_packets
//! # alert: sustained above what this Exit is willing to donate
//! avg_over_time(hopr_pix_predeposit_exposure_packets[15m]) > 1e6
//! ```
//!
//! *Where the lifecycle is stuck.* Cycles piling up in one phase name the stalled step — commitments
//! not arriving, deposits not clearing, or recovery not progressing:
//!
//! ```promql
//! sum by (phase) (hopr_pix_cycles_active)
//! ```
//!
//! *Live-cycle memory utilisation, against capacity refusals.* The pair is the point: a node at its
//! ceiling *and* refusing peers is under-provisioned, whereas refusals without utilisation are a
//! misconfigured `max_live_cycle_bytes`:
//!
//! ```promql
//! hopr_pix_live_cycle_bytes
//! rate(hopr_pix_admission_rejections_total{reason="live_cycle_capacity"}[5m])
//! ```
//!
//! *Healthy-but-full versus bad peers.* Splitting refusals by reason is what makes the two
//! distinguishable, since both are `NoSlotsAvailable` on the wire:
//!
//! ```promql
//! sum by (reason) (rate(hopr_pix_admission_rejections_total[5m]))
//! ```
//!
//! *Cycle outcome rate.* Raw counters, so the ratio is the dashboard's to compute — a continuously
//! averaged ratio would hide the volume it was computed over:
//!
//! ```promql
//! sum by (event) (rate(hopr_pix_cycles_total[10m]))
//! rate(hopr_pix_cycles_total{event="failed"}[10m])
//!   / rate(hopr_pix_cycles_total{event="requested"}[10m])
//! ```
//!
//! *Closure rate by reason*, from the existing bounded per-reason counter:
//!
//! ```promql
//! sum by (reason) (rate(hopr_session_pix_closures_total[10m]))
//! ```
//!
//! *Reservation leaks.* The two cumulative byte counters must converge once the live set drains; a
//! permanent gap with no live cycles is a reservation that was never returned:
//!
//! ```promql
//! hopr_pix_cycle_bytes_reserved_total - hopr_pix_cycle_bytes_released_total
//!   # should equal hopr_pix_live_cycle_bytes
//! ```

use crate::supervision::{
    GateBlockReason,
    telemetry::{PixAdmissionRejection, PixCycleEvent, PixCyclePhase, PixGateBlock, PixGateMode},
};

lazy_static::lazy_static! {
    static ref METRIC_PIX_SESSIONS_ACTIVE: hopr_api::types::telemetry::MultiGauge = hopr_api::types::telemetry::MultiGauge::new(
        "hopr_pix_sessions_active",
        "Incoming PIX sessions currently supervised by this Exit, by front-gate mode",
        &["gate_mode"]
    ).unwrap();
    static ref METRIC_PIX_CYCLES_ACTIVE: hopr_api::types::telemetry::MultiGauge = hopr_api::types::telemetry::MultiGauge::new(
        "hopr_pix_cycles_active",
        "SSA cycles currently retained by PIX supervisors, by phase (excludes recovered tombstones)",
        &["phase"]
    ).unwrap();
    static ref METRIC_PIX_PREDEPOSIT_EXPOSURE: hopr_api::types::telemetry::SimpleGauge = hopr_api::types::telemetry::SimpleGauge::new(
        "hopr_pix_predeposit_exposure_packets",
        "Packets served by currently unfunded PIX fronts and not yet converted into funded service"
    ).unwrap();
    static ref METRIC_PIX_LIVE_CYCLE_BYTES: hopr_api::types::telemetry::SimpleGauge = hopr_api::types::telemetry::SimpleGauge::new(
        "hopr_pix_live_cycle_bytes",
        "Reconstructor-cycle bytes currently reserved against the node's max_live_cycle_bytes"
    ).unwrap();
    static ref METRIC_PIX_CYCLE_BYTES_RESERVED: hopr_api::types::telemetry::SimpleCounter = hopr_api::types::telemetry::SimpleCounter::new(
        "hopr_pix_cycle_bytes_reserved_total",
        "Reconstructor-cycle bytes ever reserved at PIX session admission"
    ).unwrap();
    static ref METRIC_PIX_CYCLE_BYTES_RELEASED: hopr_api::types::telemetry::SimpleCounter = hopr_api::types::telemetry::SimpleCounter::new(
        "hopr_pix_cycle_bytes_released_total",
        "Reconstructor-cycle bytes ever returned to the node's live-cycle budget"
    ).unwrap();
    static ref METRIC_PIX_CYCLES_TOTAL: hopr_api::types::telemetry::MultiCounter = hopr_api::types::telemetry::MultiCounter::new(
        "hopr_pix_cycles_total",
        "SSA cycle lifecycle transitions observed by PIX supervisors, by event",
        &["event"]
    ).unwrap();
    static ref METRIC_PIX_ADMISSION_REJECTIONS: hopr_api::types::telemetry::MultiCounter = hopr_api::types::telemetry::MultiCounter::new(
        "hopr_pix_admission_rejections_total",
        "Incoming PIX sessions refused before establishment, by reason",
        &["reason"]
    ).unwrap();
    static ref METRIC_PIX_EGRESS_PACKETS: hopr_api::types::telemetry::MultiCounter = hopr_api::types::telemetry::MultiCounter::new(
        "hopr_pix_egress_packets_total",
        "Exit-to-Entry data packets admitted by PIX egress gates, by what paid for them",
        &["mode"]
    ).unwrap();
    static ref METRIC_PIX_GATE_BLOCKS: hopr_api::types::telemetry::MultiCounter = hopr_api::types::telemetry::MultiCounter::new(
        "hopr_pix_gate_blocks_total",
        "PIX egress block episodes entered, by reason (counted once per episode, not per refused packet)",
        &["reason"]
    ).unwrap();
    static ref METRIC_PIX_GATE_BLOCK_SECONDS: hopr_api::types::telemetry::MultiHistogram = hopr_api::types::telemetry::MultiHistogram::new(
        "hopr_pix_gate_block_seconds",
        "How long PIX egress stayed blocked, from first refusal until service resumed or the session closed",
        vec![0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 15.0, 60.0, 300.0],
        &["reason"]
    ).unwrap();
}

/// Moves `hopr_pix_sessions_active` for one gate mode by a signed delta.
pub(crate) fn add_sessions_active(mode: PixGateMode, delta: i64) {
    METRIC_PIX_SESSIONS_ACTIVE.increment(&[mode.to_string().as_str()], delta as f64);
}

/// Moves `hopr_pix_cycles_active` for one phase by a signed delta.
pub(crate) fn add_cycles_active(phase: PixCyclePhase, delta: i64) {
    METRIC_PIX_CYCLES_ACTIVE.increment(&[phase.to_string().as_str()], delta as f64);
}

/// Moves `hopr_pix_predeposit_exposure_packets` by a signed delta.
pub(crate) fn add_predeposit_exposure(delta: i128) {
    METRIC_PIX_PREDEPOSIT_EXPOSURE.increment(delta as f64);
}

/// Counts `count` cycles undergoing one lifecycle transition.
pub(crate) fn add_cycles_total(event: PixCycleEvent, count: u64) {
    METRIC_PIX_CYCLES_TOTAL.increment_by(&[event.to_string().as_str()], count);
}

/// Counts one incoming PIX Session refused before establishment.
pub(crate) fn record_admission_rejection(reason: PixAdmissionRejection) {
    METRIC_PIX_ADMISSION_REJECTIONS.increment(&[reason.to_string().as_str()]);
}

/// Counts a reservation of `bytes` against the node's live-cycle budget.
///
/// The live gauge is moved *by* `bytes` rather than set to the reserving caller's view of the
/// outstanding total. Session initiations are processed concurrently, so two reservations can
/// compute their totals and publish them in either order — and a `set` of the older total would
/// leave the gauge disagreeing with the budget it is meant to describe until the next mutation
/// happened to correct it. An add commutes, so the gauge is right whatever the order.
///
/// That makes the gauge a sum of its own history rather than a reading of the atomic, which is
/// sound only because the two mutations are exactly paired: `CycleBudgetReservation::release` is
/// idempotent and `Drop`-backed, so every reservation is returned exactly once. The cumulative
/// counters are what let an operator check that from outside — see the module's query on leaks.
pub(crate) fn record_cycle_bytes_reserved(bytes: u64) {
    METRIC_PIX_CYCLE_BYTES_RESERVED.increment_by(bytes);
    METRIC_PIX_LIVE_CYCLE_BYTES.increment(bytes as f64);
}

/// Counts `count` packets admitted by a PIX egress gate in one mode.
pub(crate) fn add_egress_packets(mode: PixGateMode, count: u64) {
    METRIC_PIX_EGRESS_PACKETS.increment_by(&[mode.to_string().as_str()], count);
}

/// Counts one block episode beginning.
pub(crate) fn record_gate_block(reason: PixGateBlock) {
    METRIC_PIX_GATE_BLOCKS.increment(&[reason.to_string().as_str()]);
}

/// Records how long one block episode lasted.
///
/// Only the two resumable reasons are observed. A `closed` gate never resumes, so its "duration"
/// would be the interval between a Session being torn down and its last writer noticing — which
/// says nothing about egress pressure and would drag the histogram's tail for a reason that is not
/// a stall at all.
pub(crate) fn record_gate_block_duration(reason: GateBlockReason, seconds: f64) {
    METRIC_PIX_GATE_BLOCK_SECONDS.observe(&[reason.to_string().as_str()], seconds);
}

/// Counts `bytes` returned to the node's live-cycle budget.
pub(crate) fn record_cycle_bytes_released(bytes: u64) {
    METRIC_PIX_CYCLE_BYTES_RELEASED.increment_by(bytes);
    METRIC_PIX_LIVE_CYCLE_BYTES.decrement(bytes as f64);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every PIX aggregate must be free of identifying labels.
    ///
    /// The deny-list is the point rather than the allow-list: a future instrument added to this
    /// module with a `session_id`, an SSA index or a peer address would reintroduce exactly the
    /// unbounded cardinality (#8305) these aggregates exist to avoid, and it would do so silently —
    /// the series would look correct until the node had seen ~2000 of whatever it names.
    #[test]
    fn no_aggregate_is_labelled_by_an_identifier() {
        const DENIED: &[&str] = &[
            "session_id",
            "ssa_id",
            "ssa_index",
            "pseudonym",
            "peer",
            "peerid",
            "address",
            "safe_address",
            "deposit_address",
            "target",
            "error",
        ];

        let labelled: Vec<(String, Vec<&str>)> = vec![
            (METRIC_PIX_SESSIONS_ACTIVE.name(), METRIC_PIX_SESSIONS_ACTIVE.labels()),
            (METRIC_PIX_CYCLES_ACTIVE.name(), METRIC_PIX_CYCLES_ACTIVE.labels()),
            (METRIC_PIX_CYCLES_TOTAL.name(), METRIC_PIX_CYCLES_TOTAL.labels()),
            (
                METRIC_PIX_ADMISSION_REJECTIONS.name(),
                METRIC_PIX_ADMISSION_REJECTIONS.labels(),
            ),
        ];

        for (name, labels) in labelled {
            for label in labels {
                assert!(
                    !DENIED.contains(&label),
                    "{name} is labelled by {label:?}, which is unbounded"
                );
            }
        }
    }

    #[test]
    fn aggregates_are_exported_through_hopr_metrics() {
        add_sessions_active(PixGateMode::Funded, 3);
        add_cycles_active(PixCyclePhase::Recovering, 2);
        add_cycles_total(PixCycleEvent::Requested, 5);
        record_admission_rejection(PixAdmissionRejection::LiveCycleCapacity);
        record_cycle_bytes_reserved(1024);

        let text = hopr_api::types::telemetry::gather_all_metrics().expect("must gather metrics");

        // Only the *presence* of a series with the right label is asserted, not its value: the
        // registry is process-wide and shared with every other test in this binary, so an exact
        // figure here would be a cross-test dependency rather than a fact about this module.
        for expected in [
            "hopr_pix_sessions_active{gate_mode=\"funded\"}",
            "hopr_pix_cycles_active{phase=\"recovering\"}",
            "hopr_pix_cycles_total{event=\"requested\"}",
            "hopr_pix_admission_rejections_total{reason=\"live_cycle_capacity\"}",
            "hopr_pix_live_cycle_bytes",
            "hopr_pix_cycle_bytes_reserved_total",
        ] {
            assert!(text.contains(expected), "{expected} was not exported:\n{text}");
        }
    }
}
