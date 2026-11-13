//! Node-level instruments for the packets the Exit pays for on a peer's behalf.
//!
//! The bookkeeping lives in [`crate::exit_control`] and is compiled unconditionally; this module is
//! only the emission boundary, taking closed enums so that no call site can invent a label value.
//!
//! # Prefix
//!
//! These use `hopr_exit_*` rather than `hopr_session_*` for the reason `hopr_pix_*` does: the
//! `hopr_session_*` prefix is routed to the OTLP-only meter provider and does not appear on the
//! Prometheus `/metrics` endpoint (see `OTLP.md`), while an Exit operator wants to alert on this one.
//!
//! # Metric contract
//!
//! | Metric | Type | Unit | Updated at | Labels |
//! | --- | --- | --- | --- | --- |
//! | `hopr_exit_control_packets_total` | MultiCounter | packets | a refusal is sent, dropped for want of budget, or fails to send; an admitted Session's report, fill, establishment, SSA request, SSA re-send, commitment retransmission request or session error is sent or fails to send | `kind` = one per [`ExitControlKind`] (`refusal_*`, `surb_report`, `pix_fill`, `session_established`, `ssa_request`, `ssa_request_resend`, `commitment_retransmission_request`, `session_error`), `session` = `pix\|plain`, `outcome` = `sent\|dropped_budget\|send_failed` |
//!
//! This counts packets and not their cost. The SURB a reply travels over, and with it the path
//! length and the ticket, is chosen later in the packet pipeline, so the spend cannot be known here.
//!
//! # Operator queries
//!
//! *Refusals the Exit is not answering.* A sustained rate here means the refusal budget is spent,
//! which is what a flood of requests looks like from this side:
//!
//! ```promql
//! sum by (kind) (rate(hopr_exit_control_packets_total{outcome="dropped_budget"}[5m]))
//! ```
//!
//! *What an idle Session costs.* Reports on Sessions that move no data fall as the idle backoff takes hold; a
//! rate that stays near one per base period per Session means it is off or the Sessions are not idle:
//!
//! ```promql
//! sum by (session) (rate(hopr_exit_control_packets_total{kind="surb_report",outcome="sent"}[5m]))
//! ```
//!
//! *Share of refusals answered.* Near one is an Exit that is only occasionally refusing; a falling
//! value is one being asked to refuse more than it will pay to answer:
//!
//! ```promql
//! sum(rate(hopr_exit_control_packets_total{kind=~"refusal_.*",outcome="sent"}[5m]))
//!   / sum(rate(hopr_exit_control_packets_total{kind=~"refusal_.*"}[5m]))
//! ```

use crate::exit_control::{ExitControlKind, ExitControlOutcome, ExitSessionKind};

lazy_static::lazy_static! {
    static ref METRIC_EXIT_CONTROL_PACKETS: hopr_api::types::telemetry::MultiCounter = hopr_api::types::telemetry::MultiCounter::new(
        "hopr_exit_control_packets_total",
        "Packets the Exit sent, or declined to send, on a peer's behalf, by kind, Session kind and outcome",
        &["kind", "session", "outcome"]
    ).unwrap();
}

/// Counts one packet of `kind` for a `session`, with what became of it.
pub(crate) fn record(kind: ExitControlKind, session: ExitSessionKind, outcome: ExitControlOutcome) {
    METRIC_EXIT_CONTROL_PACKETS.increment(&[
        kind.to_string().as_str(),
        session.to_string().as_str(),
        outcome.to_string().as_str(),
    ]);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// No label of this instrument may identify a peer or a Session.
    ///
    /// A deny-list, like the one for the PIX aggregates: an identifier added as a label later
    /// would reintroduce unbounded cardinality (#8305) without any series looking wrong at first.
    #[test]
    fn no_exit_control_series_is_labelled_by_an_identifier() {
        const DENIED: &[&str] = &[
            "session_id",
            "pseudonym",
            "peer",
            "peerid",
            "address",
            "target",
            "challenge",
            "error",
        ];

        for label in METRIC_EXIT_CONTROL_PACKETS.labels() {
            assert!(
                !DENIED.contains(&label),
                "hopr_exit_control_packets_total is labelled by {label:?}, which is unbounded"
            );
        }
    }

    #[test]
    fn exit_control_packets_are_exported_through_hopr_metrics() -> anyhow::Result<()> {
        use anyhow::Context;

        record(
            ExitControlKind::RefusalNoSessionSlot,
            ExitSessionKind::Pix,
            ExitControlOutcome::DroppedBudget,
        );

        let text = hopr_api::types::telemetry::gather_all_metrics().context("must gather metrics")?;

        // Each label pair is asserted on its own, since the order they are rendered in is not
        // guaranteed, and only presence is asserted: the registry is shared with the other tests.
        let line = text
            .lines()
            .find(|line| {
                line.starts_with("hopr_exit_control_packets_total{")
                    && line.contains("kind=\"refusal_no_session_slot\"")
                    && line.contains("session=\"pix\"")
                    && line.contains("outcome=\"dropped_budget\"")
            })
            .with_context(|| format!("the series was not exported:\n{text}"))?;
        assert!(!line.is_empty());
        Ok(())
    }
}
