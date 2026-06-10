//! Multi-objective channel selector.
//!
//! Ranks candidates along four orthogonal axes — latency, trust, stake, and
//! anonymity — and combines them into a single `final_score`:
//!
//! ```text
//! utility(p)     = w_lat·latency_score + w_trust·trust_score + w_stake·stake_score
//! penalty(p)     = w_anon · bucket_coverage(p.cell)
//! final_score(p) = utility(p) − penalty(p)
//! ```
//!
//! The anonymity penalty is proportional to how crowded the candidate's
//! `(latency, subnet)` cell already is among open channels — penalising dense
//! cells on open (avoid clustering) and favouring them on close (prune first).

use std::collections::{HashMap, HashSet};

use async_trait::async_trait;
use hopr_api::types::{crypto::prelude::OffchainPublicKey, internal::prelude::ChannelId, primitive::prelude::Address};
use tracing::debug;

use super::{
    CloseCandidate, OpenCandidate, Selector, SelectorContext, SignalSet,
    bucket::{BucketCell, LatencyBucket},
    default::DefaultSelector,
    subnet::SubnetBucket,
};
use crate::channel_lifecycle::config::MultiObjectiveSelectorConfig;

pub struct MultiObjectiveSelector {
    cfg: MultiObjectiveSelectorConfig,
}

impl MultiObjectiveSelector {
    pub fn new(cfg: MultiObjectiveSelectorConfig) -> Self {
        Self { cfg }
    }

    fn latency_score(candidate: &OpenCandidate) -> f64 {
        match LatencyBucket::from_latency(candidate.edge_info.average_latency) {
            LatencyBucket::Fast => 1.0,
            LatencyBucket::Medium => 0.75,
            LatencyBucket::Slow => 0.5,
            LatencyBucket::VerySlow => 0.0,
        }
    }

    fn latency_score_from_close(candidate: &CloseCandidate) -> f64 {
        match LatencyBucket::from_latency(candidate.edge_info.average_latency) {
            LatencyBucket::Fast => 1.0,
            LatencyBucket::Medium => 0.75,
            LatencyBucket::Slow => 0.5,
            LatencyBucket::VerySlow => 0.0,
        }
    }

    fn trust_score_open(candidate: &OpenCandidate, cfg: &MultiObjectiveSelectorConfig) -> f64 {
        let w = &cfg.weights;
        w.trust_probe * candidate.edge_info.probe_success_rate
            + w.trust_ack * candidate.edge_info.ack_rate.unwrap_or(0.0)
            + w.trust_ticket * candidate.ticket_score
    }

    fn trust_score_close(candidate: &CloseCandidate, cfg: &MultiObjectiveSelectorConfig) -> f64 {
        let w = &cfg.weights;
        w.trust_probe * candidate.edge_info.probe_success_rate
            + w.trust_ack * candidate.edge_info.ack_rate.unwrap_or(0.0)
            + w.trust_ticket * candidate.ticket_score
    }

    fn final_score_open(
        candidate: &OpenCandidate,
        ctx: &SelectorContext<'_>,
        cfg: &MultiObjectiveSelectorConfig,
    ) -> f64 {
        let w = &cfg.weights;
        let lat = Self::latency_score(candidate);
        let trust = Self::trust_score_open(candidate, cfg);
        let stake = ctx.stake_view.score(&candidate.addr);
        let utility = w.latency * lat + w.trust * trust + w.stake * stake;
        let cell = BucketCell(
            LatencyBucket::from_latency(candidate.edge_info.average_latency),
            candidate.subnet.clone(),
        );
        let penalty = w.anonymity * ctx.bucket_view.bucket_coverage(&cell);
        utility - penalty
    }

    fn final_score_close(
        candidate: &CloseCandidate,
        ctx: &SelectorContext<'_>,
        cfg: &MultiObjectiveSelectorConfig,
    ) -> f64 {
        let w = &cfg.weights;
        let lat = Self::latency_score_from_close(candidate);
        let trust = Self::trust_score_close(candidate, cfg);
        let stake = ctx.stake_view.score(&candidate.channel.destination);
        let utility = w.latency * lat + w.trust * trust + w.stake * stake;
        let penalty = if let Some(cell) = ctx.bucket_view.cell_for(candidate.channel.get_id()) {
            w.anonymity * ctx.bucket_view.bucket_coverage(cell)
        } else {
            0.0
        };
        utility - penalty
    }
}

#[async_trait]
impl Selector for MultiObjectiveSelector {
    fn required_signals(&self) -> SignalSet {
        if self.cfg.weights.stake > 0.0 {
            SignalSet::STAKE
        } else {
            SignalSet::empty()
        }
    }

    async fn select_closes(&self, ctx: &SelectorContext<'_>) -> Vec<ChannelId> {
        let k = self.cfg.k_floor;

        let mut scored: Vec<(&CloseCandidate, f64)> = ctx
            .close_candidates
            .iter()
            .filter(|c| DefaultSelector::should_close(c, ctx.cfg, ctx.start_epoch_elapsed))
            .filter(|c| {
                // K-floor veto: refuse to close the sole occupant of any known cell.
                // Unknown-subnet channels are never vetoed by this guard.
                if let Some(cell) = ctx.bucket_view.cell_for(c.channel.get_id()) {
                    if matches!(cell.1, SubnetBucket::Unknown) {
                        return true;
                    }
                    let count = ctx.bucket_view.cell_count(cell);
                    if count <= 1 {
                        debug!(
                            dest = %c.channel.destination,
                            cell = ?cell,
                            "channel-lifecycle: k-floor veto — sole occupant of bucket cell"
                        );
                        return false;
                    }
                }
                true
            })
            .map(|c| {
                let score = Self::final_score_close(c, ctx, &self.cfg);
                (c, score)
            })
            .collect();

        // Sort ascending: lowest score = highest close priority
        scored.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        // Simulate closes and enforce k-floor: if closing a channel would leave its cell
        // with zero occupants (among the channels NOT yet scheduled for close), veto it.
        let mut simulated_counts: HashMap<BucketCell, usize> = HashMap::new();
        for c in ctx.close_candidates.iter() {
            if let Some(cell) = ctx.bucket_view.cell_for(c.channel.get_id()) {
                *simulated_counts.entry(cell.clone()).or_insert(0) += 1;
            }
        }

        let mut result = Vec::new();
        for (c, score) in &scored {
            if result.len() >= self.cfg.close_per_tick {
                break;
            }
            let id = c.channel.get_id();
            if let Some(cell) = ctx
                .bucket_view
                .cell_for(id)
                .filter(|cell| !matches!(cell.1, SubnetBucket::Unknown))
            {
                let remaining = simulated_counts.get(cell).copied().unwrap_or(0);
                if remaining <= k {
                    // This cell is at or below floor; closing would breach it.
                    continue;
                }
                *simulated_counts.entry(cell.clone()).or_insert(1) -= 1;
            }
            debug!(
                dest = %c.channel.destination,
                score,
                "channel-lifecycle: multi-objective close candidate"
            );
            result.push(*id);
        }
        result
    }

    async fn select_opens(&self, ctx: &SelectorContext<'_>) -> Vec<(Address, OffchainPublicKey)> {
        let k = self.cfg.k_floor;
        let limit = self.cfg.open_per_tick.min(ctx.deficit);

        // Score all candidates by utility (without anonymity penalty) for fill-k ordering.
        let mut all_scored: Vec<(&OpenCandidate, f64)> = ctx
            .open_candidates
            .iter()
            .map(|c| (c, Self::final_score_open(c, ctx, &self.cfg)))
            .collect();
        all_scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let mut result: Vec<(Address, OffchainPublicKey)> = Vec::new();
        let mut picked: HashSet<Address> = HashSet::new();

        // Stage 1 — Fill-k sweep: for each (latency, subnet) cell with count < k_floor,
        // force-pick the highest-utility candidate that would land in that cell.
        // Unknown-subnet candidates are excluded from the floor enforcement.
        if k > 0 {
            // Track cells we've already attempted to fill this sweep to avoid double-filling.
            let mut attempted_cells: HashSet<BucketCell> = HashSet::new();
            // Simulate how many times each cell will be filled as we pick.
            let mut fill_counts: HashMap<BucketCell, usize> = HashMap::new();

            for (c, _score) in &all_scored {
                if result.len() >= limit {
                    break;
                }
                if matches!(c.subnet, SubnetBucket::Unknown) {
                    continue;
                }
                let cell = BucketCell(
                    LatencyBucket::from_latency(c.edge_info.average_latency),
                    c.subnet.clone(),
                );
                if attempted_cells.contains(&cell) {
                    continue;
                }
                attempted_cells.insert(cell.clone());

                let existing = ctx.bucket_view.cell_count(&cell);
                let already_filling = fill_counts.get(&cell).copied().unwrap_or(0);
                if existing + already_filling < k {
                    // Pick the first (highest-utility) candidate in this underrepresented cell.
                    if let Some((candidate, score)) = all_scored.iter().find(|(c2, _)| {
                        !picked.contains(&c2.addr)
                            && BucketCell(
                                LatencyBucket::from_latency(c2.edge_info.average_latency),
                                c2.subnet.clone(),
                            ) == cell
                    }) {
                        debug!(
                            addr = %candidate.addr,
                            score,
                            cell = ?cell,
                            "channel-lifecycle: fill-k sweep open"
                        );
                        picked.insert(candidate.addr);
                        *fill_counts.entry(cell).or_insert(0) += 1;
                        result.push((candidate.addr, candidate.offchain_key));
                    }
                }
            }
        }

        // Stage 2 — Utility ranking: fill remaining slots from highest final_score desc.
        for (c, score) in &all_scored {
            if result.len() >= limit {
                break;
            }
            if picked.contains(&c.addr) {
                continue;
            }
            debug!(
                addr = %c.addr,
                score,
                "channel-lifecycle: multi-objective open candidate"
            );
            picked.insert(c.addr);
            result.push((c.addr, c.offchain_key));
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use hopr_api::types::{
        crypto::prelude::{Keypair, OffchainKeypair, OffchainPublicKey},
        primitive::prelude::{Address, BytesRepresentable},
    };

    use super::*;
    use crate::channel_lifecycle::{
        ChannelLifecycleConfig,
        config::MultiObjectiveSelectorConfig,
        selector::{BucketView, PeerEdgeInfo, StakeView, SubnetBucket},
    };

    fn mk_selector(cfg: MultiObjectiveSelectorConfig) -> MultiObjectiveSelector {
        MultiObjectiveSelector::new(cfg)
    }

    fn addr(seed: u8) -> Address {
        [seed; Address::SIZE].into()
    }

    fn offchain_key(seed: u8) -> OffchainPublicKey {
        *OffchainKeypair::from_secret(&[seed; 32]).expect("test key").public()
    }

    fn mk_candidate(
        a: Address,
        ok: OffchainPublicKey,
        latency_ms: Option<u64>,
        probe_rate: f64,
        ticket_score: f64,
        subnet_id: u8,
    ) -> OpenCandidate {
        OpenCandidate {
            addr: a,
            offchain_key: ok,
            edge_info: PeerEdgeInfo {
                average_latency: latency_ms.map(Duration::from_millis),
                probe_success_rate: probe_rate,
                ack_rate: Some(probe_rate),
                edge_score: Some(probe_rate),
                last_update: Duration::from_secs(1),
            },
            ticket_score,
            subnet: SubnetBucket::V4Prefix([subnet_id, 0, 0]),
        }
    }

    #[test]
    fn required_signals_stake_nonzero() {
        let cfg = MultiObjectiveSelectorConfig::low_latency();
        assert!(cfg.weights.stake > 0.0);
        let sel = mk_selector(cfg);
        assert!(sel.required_signals().contains(SignalSet::STAKE));
    }

    #[test]
    fn required_signals_stake_zero() {
        let mut cfg = MultiObjectiveSelectorConfig::low_latency();
        cfg.weights.stake = 0.0;
        let sel = mk_selector(cfg);
        assert_eq!(sel.required_signals(), SignalSet::empty());
    }

    #[tokio::test]
    async fn low_latency_profile_prefers_fast_peer() {
        let sel = mk_selector(MultiObjectiveSelectorConfig::low_latency());
        let cfg = ChannelLifecycleConfig::default();

        let fast = mk_candidate(addr(1), offchain_key(1), Some(50), 0.8, 0.5, 1);
        let slow = mk_candidate(addr(2), offchain_key(2), Some(300), 0.9, 0.9, 2);

        let ctx = SelectorContext {
            cfg: &cfg,
            deficit: 2,
            open_candidates: &[fast.clone(), slow.clone()],
            close_candidates: &[],
            start_epoch_elapsed: Duration::from_secs(600),
            bucket_view: BucketView::empty(),
            stake_view: StakeView::empty(),
        };

        let opens = sel.select_opens(&ctx).await;
        assert_eq!(opens.len(), 2);
        // LowLatency weights latency heavily: fast peer should rank first
        assert_eq!(opens[0].0, fast.addr);
    }

    #[tokio::test]
    async fn open_per_tick_caps_output() {
        let mut cfg = MultiObjectiveSelectorConfig::low_latency();
        cfg.open_per_tick = 1;
        let sel = mk_selector(cfg);
        let lc_cfg = ChannelLifecycleConfig::default();

        let candidates: Vec<OpenCandidate> = (0u8..5)
            .map(|i| mk_candidate(addr(i), offchain_key(i), Some(50 + i as u64 * 10), 0.8, 0.5, i))
            .collect();

        let ctx = SelectorContext {
            cfg: &lc_cfg,
            deficit: 5,
            open_candidates: &candidates,
            close_candidates: &[],
            start_epoch_elapsed: Duration::from_secs(600),
            bucket_view: BucketView::empty(),
            stake_view: StakeView::empty(),
        };

        let opens = sel.select_opens(&ctx).await;
        assert_eq!(opens.len(), 1);
    }

    #[tokio::test]
    async fn deficit_caps_output_below_open_per_tick() {
        let sel = mk_selector(MultiObjectiveSelectorConfig::low_latency()); // open_per_tick=4
        let lc_cfg = ChannelLifecycleConfig::default();

        let candidates: Vec<OpenCandidate> = (0u8..5)
            .map(|i| mk_candidate(addr(i), offchain_key(i), Some(50), 0.8, 0.5, i))
            .collect();

        let ctx = SelectorContext {
            cfg: &lc_cfg,
            deficit: 2, // only 2 slots available even though open_per_tick=4
            open_candidates: &candidates,
            close_candidates: &[],
            start_epoch_elapsed: Duration::from_secs(600),
            bucket_view: BucketView::empty(),
            stake_view: StakeView::empty(),
        };

        let opens = sel.select_opens(&ctx).await;
        assert_eq!(opens.len(), 2);
    }

    #[tokio::test]
    async fn anonymity_penalty_discourages_crowded_cell() {
        // Two candidates in different subnets, same latency/trust.
        // Bucket view already has 3 channels in subnet 1's cell — subnet 2 should rank first.
        use crate::channel_lifecycle::selector::bucket::{BucketCell, LatencyBucket};
        use hopr_api::types::internal::prelude::ChannelId;
        use std::collections::HashMap;

        let addr1 = addr(10);
        let addr2 = addr(20);

        let c1 = mk_candidate(addr1, offchain_key(10), Some(50), 0.8, 0.5, 1); // crowded bucket
        let c2 = mk_candidate(addr2, offchain_key(20), Some(50), 0.8, 0.5, 2); // empty bucket

        let crowded_cell = BucketCell(LatencyBucket::Fast, SubnetBucket::V4Prefix([1, 0, 0]));
        let cells: HashMap<ChannelId, BucketCell> = (0u8..3)
            .map(|i| (ChannelId::create(&[&[i]]), crowded_cell.clone()))
            .collect();
        let bucket_view = BucketView::new(cells);

        let mut weights_cfg = MultiObjectiveSelectorConfig::dispersed();
        weights_cfg.open_per_tick = 2;
        let sel = mk_selector(weights_cfg);
        let lc_cfg = ChannelLifecycleConfig::default();

        let ctx = SelectorContext {
            cfg: &lc_cfg,
            deficit: 2,
            open_candidates: &[c1.clone(), c2.clone()],
            close_candidates: &[],
            start_epoch_elapsed: Duration::from_secs(600),
            bucket_view,
            stake_view: StakeView::empty(),
        };

        let opens = sel.select_opens(&ctx).await;
        assert_eq!(opens.len(), 2);
        // Dispersed profile has high w_anon — empty-bucket peer should rank first
        assert_eq!(opens[0].0, addr2);
    }

    // ── PR4: k-floor tests ───────────────────────────────────────────────────

    /// fill-k sweep: candidate in an underrepresented cell gets picked even when
    /// its final_score is lower than a candidate in an already-populated cell.
    #[tokio::test]
    async fn fill_k_forces_underrepresented_cell_first() {
        use hopr_api::types::internal::prelude::ChannelId;
        use std::collections::HashMap;

        // Two candidates: one in an already-populated cell (subnet 1, count=2),
        // one in an empty cell (subnet 2, count=0).  LowLatency k_floor=2, but
        // cell 1 already has 2 → floor satisfied; cell 2 has 0 → needs filling.
        // Even though the subnet-1 candidate has higher latency-utility, the sweep
        // should pick the subnet-2 candidate first.

        // Build bucket_view with 2 channels in cell (Fast, subnet 1)
        let cell_1 = BucketCell(LatencyBucket::Fast, SubnetBucket::V4Prefix([1, 0, 0]));
        let existing_cells: HashMap<ChannelId, BucketCell> =
            (0u8..2).map(|i| (ChannelId::create(&[&[i]]), cell_1.clone())).collect();
        let bucket_view = BucketView::new(existing_cells);

        // Candidate A: subnet 1 (already 2 in bucket → at k_floor)
        // Candidate B: subnet 2 (0 in bucket → below k_floor)
        let a = mk_candidate(addr(1), offchain_key(1), Some(50), 0.95, 0.9, 1); // high utility, cell at floor
        let b = mk_candidate(addr(2), offchain_key(2), Some(50), 0.3, 0.1, 2); // low utility, cell below floor

        let mut cfg = MultiObjectiveSelectorConfig::low_latency(); // k_floor=2
        cfg.open_per_tick = 2;
        let sel = mk_selector(cfg);
        let lc_cfg = ChannelLifecycleConfig::default();

        let ctx = SelectorContext {
            cfg: &lc_cfg,
            deficit: 2,
            open_candidates: &[a.clone(), b.clone()],
            close_candidates: &[],
            start_epoch_elapsed: Duration::from_secs(600),
            bucket_view,
            stake_view: StakeView::empty(),
        };

        let opens = sel.select_opens(&ctx).await;
        assert_eq!(opens.len(), 2);
        // Fill-k sweep must pick b first (its cell is below k_floor)
        assert_eq!(
            opens[0].0,
            addr(2),
            "fill-k should pick the underrepresented cell first"
        );
    }

    /// close veto: sole occupant of a cell must NOT be returned as a close target.
    #[tokio::test]
    async fn k_floor_veto_blocks_sole_occupant_close() {
        use hopr_api::types::internal::prelude::{ChannelEntry, ChannelId, ChannelStatus};
        use hopr_api::types::primitive::prelude::HoprBalance;
        use std::collections::HashMap;
        use std::time::Duration;

        let dest = addr(5);

        // Build the channel first so we can get the correct channel ID.
        let ch = ChannelEntry::builder()
            .between(addr(0), dest)
            .balance(HoprBalance::new_base(1))
            .ticket_index(0)
            .status(ChannelStatus::Open)
            .epoch(1)
            .build()
            .expect("test channel");

        // Populate bucket_view with the channel's actual ID.
        let unique_cell = BucketCell(LatencyBucket::Fast, SubnetBucket::V4Prefix([5, 0, 0]));
        let mut bucket_cells: HashMap<ChannelId, BucketCell> = HashMap::new();
        bucket_cells.insert(*ch.get_id(), unique_cell);
        let bucket_view = BucketView::new(bucket_cells);

        // Make the channel look like it should_close: balance drained
        let candidate = crate::channel_lifecycle::selector::CloseCandidate {
            channel: ch,
            offchain_key: Some(offchain_key(5)),
            edge_info: PeerEdgeInfo {
                edge_score: Some(0.0),
                last_update: Duration::from_secs(9999),
                average_latency: Some(Duration::from_millis(50)),
                probe_success_rate: 0.0,
                ack_rate: Some(0.0),
            },
            ticket_score: 0.0,
        };

        let mut mo_cfg = MultiObjectiveSelectorConfig::dispersed(); // k_floor=4
        mo_cfg.close_per_tick = 4;
        let sel = mk_selector(mo_cfg);

        let mut lc_cfg = ChannelLifecycleConfig::default();
        lc_cfg.closure.close_below_quality_score = 1.0; // force should_close=true for the candidate

        let ctx = SelectorContext {
            cfg: &lc_cfg,
            deficit: 0,
            open_candidates: &[],
            close_candidates: &[candidate],
            start_epoch_elapsed: Duration::from_secs(600),
            bucket_view,
            stake_view: StakeView::empty(),
        };

        let closes = sel.select_closes(&ctx).await;
        assert!(closes.is_empty(), "sole-occupant channel must not be closed");
    }
}
