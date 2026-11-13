//! Node-level instruments for the packets the Exit pays for on a peer's behalf. Bookkeeping is in
//! [`crate::exit_control`]; this is the emission boundary, taking closed enums for labels.
//!
//! These use `hopr_exit_*` rather than `hopr_session_*` because the latter is routed to the OTLP-only
//! provider (see `OTLP.md`), and an Exit operator wants to alert on this on `/metrics`.
//!
//! # Metric contract
//!
//! | Metric | Type | Unit | Updated at | Labels |
//! | --- | --- | --- | --- | --- |
//! | `hopr_exit_control_packets_total` | MultiCounter | packets | a refusal is sent, dropped for want of budget, or fails to send | `kind` = `refusal_*` (see [`ExitControlKind`]), `session` = `pix\|plain`, `outcome` = `sent\|dropped_budget\|send_failed` |
//!
//! It counts packets, not their cost: the SURB, and so the path length and ticket, is chosen later in
//! the packet pipeline.
//!
//! ```promql
//! sum by (kind) (rate(hopr_exit_control_packets_total{outcome="dropped_budget"}[5m]))
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

    /// No label may identify a peer or a Session: that would be unbounded cardinality (#8305).
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

        // Label order is not guaranteed, and the registry is shared, so only presence is asserted.
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
