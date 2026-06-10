//! Default selector: reproduces the original `peer_score_for` / `should_close`
//! logic verbatim, with zero behavior change from the pre-refactor pipeline.

use std::time::Duration;

use async_trait::async_trait;
use hopr_api::types::{crypto::prelude::OffchainPublicKey, internal::prelude::ChannelId, primitive::prelude::Address};
use tracing::debug;

use super::{CloseCandidate, OpenCandidate, Selector, SelectorContext};

/// Stateless selector that exactly reproduces the behavior of the original
/// `peer_score_for` + `should_close` methods.  This is the default selector;
/// all existing deployments use it unless they opt in to a different profile.
pub struct DefaultSelector;

impl DefaultSelector {
    /// Composite quality score for a candidate peer.
    ///
    /// Mirrors the original `ChannelLifecycleStrategyInner::peer_score_for`.
    fn peer_score(candidate: &OpenCandidate, cfg: &super::super::ChannelLifecycleConfig) -> f64 {
        let edge_score = candidate.edge_info.edge_score.unwrap_or(0.0);
        cfg.eligibility.peer_quality_weight * edge_score
            + cfg.eligibility.ticket_activity_weight * candidate.ticket_score
    }

    /// Returns `true` when the channel should be closed.
    ///
    /// Mirrors the original `ChannelLifecycleStrategyInner::should_close`.
    fn should_close(
        candidate: &CloseCandidate,
        cfg: &super::super::ChannelLifecycleConfig,
        start_epoch_elapsed: Duration,
    ) -> bool {
        let ch = &candidate.channel;

        if ch.balance <= cfg.closure.close_when_drained_below {
            debug!(
                dest = %ch.destination,
                balance = %ch.balance,
                threshold = %cfg.closure.close_when_drained_below,
                reason = "balance_drained",
                "channel-lifecycle: close candidate"
            );
            return true;
        }

        if candidate.offchain_key.is_none() {
            return false;
        }

        if !candidate.edge_info.has_probing_data() {
            tracing::trace!(
                dest = %ch.destination,
                "channel-lifecycle: skipping close evaluation — no graph observations yet"
            );
            return false;
        }

        let edge_score = candidate.edge_info.edge_score.unwrap_or(0.0);
        let composite_score = cfg.eligibility.peer_quality_weight * edge_score
            + cfg.eligibility.ticket_activity_weight * candidate.ticket_score;

        if composite_score < cfg.closure.close_below_quality_score {
            debug!(
                dest = %ch.destination,
                score = composite_score,
                threshold = cfg.closure.close_below_quality_score,
                reason = "low_quality_score",
                "channel-lifecycle: close candidate"
            );
            return true;
        }

        let last_update = candidate.edge_info.last_update;
        let stale = last_update > cfg.closure.close_when_peer_unseen_for;
        // `last_update` is the age of the last observation. If it is smaller
        // than `start_epoch_elapsed`, the observation was recorded after this
        // strategy instance started — the peer has been seen during this run.
        let observed_since_start = last_update < start_epoch_elapsed;
        let guard_passed = !cfg.eligibility.require_observed_since_start || observed_since_start;
        if stale && guard_passed {
            debug!(
                dest = %ch.destination,
                last_update_secs = last_update.as_secs(),
                unseen_threshold_secs = cfg.closure.close_when_peer_unseen_for.as_secs(),
                reason = "peer_stale",
                "channel-lifecycle: close candidate"
            );
            return true;
        }

        false
    }
}

#[async_trait]
impl Selector for DefaultSelector {
    fn required_signals(&self) -> super::SignalSet {
        super::SignalSet::empty()
    }

    async fn select_closes(&self, ctx: &SelectorContext<'_>) -> Vec<ChannelId> {
        ctx.close_candidates
            .iter()
            .filter(|c| Self::should_close(c, ctx.cfg, ctx.start_epoch_elapsed))
            .map(|c| *c.channel.get_id())
            .collect()
    }

    async fn select_opens(&self, ctx: &SelectorContext<'_>) -> Vec<(Address, OffchainPublicKey)> {
        let mut scored: Vec<(&OpenCandidate, f64)> = ctx
            .open_candidates
            .iter()
            .map(|c| (c, Self::peer_score(c, ctx.cfg)))
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        scored.into_iter().map(|(c, _)| (c.addr, c.offchain_key)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_selector_requires_no_signals() {
        assert_eq!(DefaultSelector.required_signals(), super::super::SignalSet::empty());
    }
}
