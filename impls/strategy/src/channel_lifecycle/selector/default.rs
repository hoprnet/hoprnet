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
    /// `quality_threshold` overrides `cfg.closure.close_below_quality_score`; pass
    /// `None` to use the config value.  `MultiObjectiveSelector` passes an adjusted
    /// value to enforce the hysteresis gap.
    pub(super) fn should_close(
        candidate: &CloseCandidate,
        cfg: &super::super::ChannelLifecycleConfig,
        start_epoch_elapsed: Duration,
        quality_threshold: Option<f64>,
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

        let effective_threshold = quality_threshold.unwrap_or(cfg.closure.close_below_quality_score);
        if composite_score < effective_threshold {
            debug!(
                dest = %ch.destination,
                score = composite_score,
                threshold = effective_threshold,
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
        super::SignalSet::default()
    }

    async fn select_closes(&self, ctx: &SelectorContext<'_>) -> Vec<ChannelId> {
        ctx.close_candidates
            .iter()
            .filter(|c| Self::should_close(c, ctx.cfg, ctx.start_epoch_elapsed, None))
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
    use std::time::Duration;

    use hopr_api::types::{
        crypto::prelude::{Keypair, OffchainKeypair, OffchainPublicKey},
        internal::prelude::{ChannelEntry, ChannelStatus},
        primitive::prelude::{Address, BytesRepresentable, HoprBalance},
    };

    use super::*;
    use crate::channel_lifecycle::{
        ChannelLifecycleConfig,
        selector::{CloseCandidate, PeerEdgeInfo},
    };

    fn addr(seed: u8) -> Address {
        [seed; Address::SIZE].into()
    }

    fn offchain_key(seed: u8) -> OffchainPublicKey {
        *OffchainKeypair::from_secret(&[seed; 32]).expect("test key").public()
    }

    fn open_channel(src: Address, dest: Address) -> ChannelEntry {
        ChannelEntry::builder()
            .between(src, dest)
            .balance(HoprBalance::new_base(10))
            .ticket_index(0)
            .status(ChannelStatus::Open)
            .epoch(1)
            .build()
            .expect("test channel")
    }

    #[test]
    fn default_selector_requires_no_signals() {
        assert_eq!(DefaultSelector.required_signals(), super::super::SignalSet::default());
    }

    /// A close candidate with `offchain_key = None` (key not resolvable from address map)
    /// must never be closed by `should_close`, regardless of other signals.
    #[test]
    fn should_close_returns_false_when_offchain_key_is_none() {
        let ch = open_channel(addr(0), addr(1));
        let candidate = CloseCandidate {
            channel: ch,
            offchain_key: None, // key not resolvable
            edge_info: PeerEdgeInfo {
                edge_score: Some(0.0), // terrible quality — would normally close
                last_update: Duration::from_secs(9999),
                average_latency: Some(Duration::from_millis(300)),
                probe_success_rate: 0.0,
                ack_rate: Some(0.0),
            },
            ticket_score: 0.0,
        };
        let cfg = ChannelLifecycleConfig::default();
        assert!(
            !DefaultSelector::should_close(&candidate, &cfg, Duration::from_secs(600), None),
            "channel without a resolvable offchain key must not be closed"
        );
    }

    /// A channel with no graph observations yet (last_update = ZERO) must not be closed.
    #[test]
    fn should_close_returns_false_when_no_probing_data() {
        let ch = open_channel(addr(0), addr(2));
        let candidate = CloseCandidate {
            channel: ch,
            offchain_key: Some(offchain_key(2)),
            edge_info: PeerEdgeInfo {
                edge_score: None,
                last_update: Duration::ZERO, // no observations at all
                average_latency: None,
                probe_success_rate: 0.0,
                ack_rate: None,
            },
            ticket_score: 0.0,
        };
        let mut cfg = ChannelLifecycleConfig::default();
        cfg.closure.close_below_quality_score = 1.0; // would close anything with data
        assert!(
            !DefaultSelector::should_close(&candidate, &cfg, Duration::from_secs(600), None),
            "channel with no graph observations must not be closed"
        );
    }

    /// A balance-drained channel must be closed regardless of other signals.
    #[test]
    fn should_close_returns_true_when_balance_drained() {
        let ch = ChannelEntry::builder()
            .between(addr(0), addr(3))
            .balance(HoprBalance::zero()) // fully drained
            .ticket_index(0)
            .status(ChannelStatus::Open)
            .epoch(1)
            .build()
            .expect("test channel");
        let candidate = CloseCandidate {
            channel: ch,
            offchain_key: Some(offchain_key(3)),
            edge_info: PeerEdgeInfo {
                edge_score: Some(1.0), // excellent quality — would normally keep open
                last_update: Duration::from_secs(10),
                average_latency: Some(Duration::from_millis(50)),
                probe_success_rate: 1.0,
                ack_rate: Some(1.0),
            },
            ticket_score: 1.0,
        };
        let cfg = ChannelLifecycleConfig::default();
        assert!(
            DefaultSelector::should_close(&candidate, &cfg, Duration::from_secs(600), None),
            "balance-drained channel must always be closed"
        );
    }
}
