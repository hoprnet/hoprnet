//! Node-level instruments for the Exit's SURB keep-alive reserve hold.
//!
//! The episode that drives them lives in [`crate::supervision::telemetry`] and is compiled
//! unconditionally; this module is only the emission boundary, exactly as [`pix`](super::pix) is
//! for the PIX aggregates. The hold it measures is not PIX-only — it sits on the Exit's data egress
//! *before* the PIX gate, so it fires for a rate-controlled Session without PIX as well.
//!
//! # Why `hopr_surb_*` and not `hopr_session_surb_*`
//!
//! `hopr-types`' `meter_for_metric` routes an instrument to a meter provider by *name prefix*, and
//! hoprd builds one provider per prefix configured in `HOPRD_METRIC_EXPORT_INTERVAL`. Such a
//! provider always gets the OTLP reader; it gets the Prometheus reader too **unless** its prefix
//! starts with `hopr_session`. So once a `hopr_session` interval override is configured — which
//! `OTLP.md` recommends — `hopr_session_*` series stop reaching `/metrics`, while every other
//! prefix keeps reaching it.
//!
//! An operator alerting on an SURB shortage reads `/metrics`, and the `hopr_surb_*` prefix is never
//! diverted to the OTLP-only provider: it reaches `/metrics` under every configuration. A
//! `hopr_surb` interval override would give these two their own provider rather than the default
//! one, and therefore their own export cadence, but not a different exporter set. They carry the
//! prefix the rest of the node-level SURB metrics already use (`hopr_surb_balancer_*`,
//! `hopr_surb_store_*`).
//!
//! # Why a `pix` label rather than no label
//!
//! A PIX Session's hold is one of two gates an Exit operator watches; a non-PIX hold is the *only*
//! SURB backpressure a rate-controlled Session has. An unlabelled counter would also conflate
//! breadth with depth: one starved Session emits one episode per held data packet, which on the
//! counter alone looks identical to a node-wide shortage. The label is a closed set — a `bool`
//! chosen at construction from the egress gate the Session was built with — so it cannot grow the
//! way a `session_id` would (#8305).
//!
//! # Not the same signal as `hopr_pix_fill_backoff_total{reason="surb_reserve"}`
//!
//! That pre-existing counter counts *keep-alives the Exit withheld* from its own PIX fill schedule
//! while the estimated SURB level is below `fill.min_surb_reserve`. These instruments count *data
//! packets the Exit held* on behalf of the Entry, on any rate-controlled Session. Both rise under an
//! SURB shortage, so summing them as one pressure signal would double-count: they answer "how much
//! of my own fill did I suppress" versus "how much of the Entry's data did I hold".
//!
//! # Metric contract
//!
//! | Metric | Type | Unit | Updated at | Labels |
//! | --- | --- | --- | --- | --- |
//! | `hopr_surb_reserve_holds_total` | MultiCounter | holds | the hold future is created, before its first poll | `pix` = `true\|false` |
//! | `hopr_surb_reserve_hold_seconds` | MultiHistogram | seconds | the hold ends (release, cancellation, Session teardown or drop before the first poll) | `pix` = `true\|false` |
//!
//! Neither is labelled by a Session: a per-Session label would stop reporting new Sessions after
//! ~2000 of them (#8305), and these are node-level pressure signals. A packet above the reserve
//! creates no episode; a packet held across many polling intervals creates one, not one per poll.
//!
//! The histogram has no upper bucket at the Session's lifetime: the hold has no deadline of its
//! own, so a Session whose Entry never refills holds until its teardown. The top buckets are
//! therefore 600 s and 1800 s, past the "minutes of a total outage" the reserve is sized for.

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
