//! Node-level instruments for the Exit's SURB keep-alive reserve hold.
//!
//! The episode lives in [`crate::supervision::telemetry`]; this is only the emission boundary, as
//! [`pix`](super::pix) is for the PIX aggregates. The hold sits on the egress *before* the PIX gate,
//! so it fires for a rate-controlled Session without PIX too.
//!
//! `hopr_surb_*` rather than `hopr_session_surb_*` because hoprd gives a `hopr_session`-prefixed
//! provider an OTLP-only reader once an interval override is configured; `hopr_surb_*` reaches
//! `/metrics` under every configuration. See `OTLP.md`.
//!
//! The `pix` label is a closed `bool` chosen from the Session's egress gate: it separates a PIX
//! hold from the non-PIX backpressure a rate-controlled Session relies on, and cannot grow the way
//! a `session_id` would (#8305). This pair counts *data packets held*; the pre-existing
//! `hopr_pix_fill_backoff_total{reason="surb_reserve"}` counts *keep-alives the Exit withheld*, so
//! summing them double-counts.
//!
//! # Metric contract
//!
//! | Metric | Type | Unit | Updated at | Labels |
//! | --- | --- | --- | --- | --- |
//! | `hopr_surb_reserve_holds_total` | MultiCounter | holds | the hold future is created, before its first poll | `pix` = `true\|false` |
//! | `hopr_surb_reserve_hold_seconds` | MultiHistogram | seconds | the hold ends (release, cancellation, Session teardown or drop before the first poll) | `pix` = `true\|false` |
//!
//! A packet above the reserve creates no episode; one held across many polls creates one. The top
//! buckets (600 s, 1800 s) cover a hold with no deadline of its own, which lasts until teardown.

lazy_static::lazy_static! {
    static ref METRIC_SURB_RESERVE_HOLDS_TOTAL: hopr_api::types::telemetry::MultiCounter = hopr_api::types::telemetry::MultiCounter::new(
        "hopr_surb_reserve_holds_total",
        "Exit data holds entered by the SURB keep-alive reserve, by whether the Session runs PIX (counted once per held packet, not per poll); distinct from hopr_pix_gate_blocks_total and from hopr_pix_fill_backoff_total, and raised for rate-controlled Sessions without PIX too",
        &["pix"]
    ).unwrap();
    static ref METRIC_SURB_RESERVE_HOLD_SECONDS: hopr_api::types::telemetry::MultiHistogram = hopr_api::types::telemetry::MultiHistogram::new(
        "hopr_surb_reserve_hold_seconds",
        "How long Exit data stayed held by the SURB keep-alive reserve, by whether the Session runs PIX, observed once per hold on release, cancellation or Session teardown",
        vec![0.05, 0.1, 0.25, 0.5, 1.0, 5.0, 15.0, 60.0, 300.0, 600.0, 1800.0],
        &["pix"]
    ).unwrap();
}

/// The `pix` label value, spelled in [`crate::supervision::telemetry`] and used here so the
/// instrument and the probe mirror cannot drift.
fn pix_label(pix: bool) -> &'static str {
    crate::supervision::telemetry::pix_label(pix)
}

/// Counts one hold of Exit data by the SURB keep-alive reserve beginning.
///
/// Counted once when the hold future is created, not once per poll, so the counter measures held
/// packets rather than how long they waited.
pub(crate) fn record_surb_reserve_hold(pix: bool) {
    METRIC_SURB_RESERVE_HOLDS_TOTAL.increment(&[pix_label(pix)]);
}

/// Observes how long one hold of Exit data by the SURB keep-alive reserve lasted.
///
/// One observation per closed episode, whether it ended by release, cancellation, Session teardown
/// or a drop before its first poll.
pub(crate) fn record_surb_reserve_hold_duration(pix: bool, seconds: f64) {
    METRIC_SURB_RESERVE_HOLD_SECONDS.observe(&[pix_label(pix)], seconds);
}
