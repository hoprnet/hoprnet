//! Anonymity-set diversity of the routes the planner selects.
//!
//! Return paths are drawn weighted-random by path value, so a session's SURBs concentrate on the
//! few best-scoring relayers even when the graph offers several. The planner already warns when the
//! *candidate pool* has collapsed to a single first relayer, but that says nothing about where the
//! draws actually go over time: a pool of ten can still serve one relayer for hours.
//!
//! This module measures that in the two dimensions an observer can use to tell sessions apart:
//!
//! - **First hop, per direction.** The first relayer of the forward path and of a return path is the only relay that
//!   learns who talks to whom, so each direction is scored on its own. A session whose forward and return path leave
//!   through the same relay exposes the pair; one that keeps both first hops varied does not.
//! - **Whole route.** A route is a forward path together with the return path of one SURB. Scoring whole routes catches
//!   a correlation the first hops cannot show, such as a stable pair of relayers that every packet reuses behind varied
//!   outer hops.
//!
//! Every score is the effective number of distinct values, `exp(H)` for the Shannon entropy `H` of
//! the distribution: 1.0 when every draw used the same value, tending to the number of distinct
//! values as the draws even out. An operator can alert on a score that stays near 1.

use std::{
    collections::{HashSet, VecDeque},
    hash::Hash,
};

use ahash::HashMap;
use hopr_api::OffchainPublicKey;

/// Draws a session needs in a dimension before its score may raise a WARN.
///
/// One draw is always a score of 1.0, and so is a second draw through the same relayer. Logging
/// from the first draw would raise a WARN/INFO pair for every session that starts, which says
/// nothing about the node and buries the case the check exists for.
const MIN_DRAWS_FOR_THRESHOLD: usize = 10;

/// Number of sessions the tracker keeps a window for.
///
/// A node serves a handful of return paths at a time, so the bound only stops a churning set of
/// destinations from growing the tracker without limit.
pub(crate) const MAX_TRACKED_SESSIONS: usize = 256;

/// One session's identity in the tracker: the return-path destination and the hop count.
///
/// The planner has no session identity of its own, and its cache is keyed the same way, so this is
/// the granularity at which its draws are told apart.
pub(crate) type DiversityKey = (OffchainPublicKey, u32);

/// The dimension a score covers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum Dimension {
    /// First relayer of the forward path.
    ForwardFirstHop,
    /// First relayer of a return path.
    ReturnFirstHop,
    /// Whole route: one forward path with the return path of one SURB.
    Route,
}

/// The pooled scores over every tracked session, one per dimension that has seen a draw.
///
/// Read by the gauge publication only, which is compiled out without telemetry.
#[cfg_attr(not(any(test, feature = "telemetry")), allow(dead_code))]
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Scores {
    pub forward_first_hop: Option<f64>,
    pub return_first_hop: Option<f64>,
    pub route: Option<f64>,
    /// Draws the pooled score of each dimension covers, zero when it has none.
    ///
    /// Published next to the score because a gauge keeps its last value after its draws are
    /// evicted: only this count says whether a score was computed from anything.
    #[cfg_attr(not(any(test, feature = "telemetry")), allow(dead_code))]
    pub forward_first_hop_draws: usize,
    #[cfg_attr(not(any(test, feature = "telemetry")), allow(dead_code))]
    pub return_first_hop_draws: usize,
    #[cfg_attr(not(any(test, feature = "telemetry")), allow(dead_code))]
    pub route_draws: usize,
}

/// Effective number of distinct values of a draw distribution: `exp(H)` over the counts.
///
/// Takes the running `Σ c·ln c` rather than the counts themselves, so a score costs a constant
/// number of operations however many values the distribution holds: the pooled scores are read on
/// the packet path, and walking their counts there would cost more the more diverse they are.
///
/// `None` when nothing was drawn: the score is undefined there, and a caller must not publish an
/// invented value.
fn effective_size(drawn: usize, distinct: usize, sum_count_log_count: f64) -> Option<f64> {
    if drawn == 0 {
        return None;
    }
    // One value is the collapsed case, and it must read exactly 1.0: the WARN and the alert
    // compare against 1.0, and the accumulated sum leaves a rounding error that a `<=` misses.
    if distinct <= 1 {
        return Some(1.0);
    }
    let total = drawn as f64;
    // H = ln N - (Σ c ln c) / N, so exp(H) = N * exp(-Σ c ln c / N).
    Some((total.ln() - sum_count_log_count / total).exp())
}

/// Change in `Σ c·ln c` when one value's count moves from `from` to `to`.
fn count_log_count_delta(from: usize, to: usize) -> f64 {
    let term = |count: usize| {
        if count == 0 {
            0.0
        } else {
            let count = count as f64;
            count * count.ln()
        }
    };
    term(to) - term(from)
}

/// The same total, with nothing behind it, where no gauge reads it.
#[cfg(not(any(test, feature = "telemetry")))]
#[derive(Debug)]
struct Pooled<K> {
    _marker: std::marker::PhantomData<K>,
}

#[cfg(not(any(test, feature = "telemetry")))]
impl<K: Copy + Eq + Hash> Default for Pooled<K> {
    fn default() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

#[cfg(not(any(test, feature = "telemetry")))]
impl<K: Copy + Eq + Hash> Pooled<K> {
    fn add(&mut self, _value: K) {}

    fn retract(&mut self, _value: K, _drawn: usize) {}

    fn score(&self) -> Option<f64> {
        None
    }

    fn draws(&self) -> usize {
        0
    }
}

/// The last `capacity` values a session drew, with their counts and `Σ c·ln c` alongside.
#[derive(Debug)]
struct DiversityWindow<K> {
    capacity: usize,
    drawn: VecDeque<K>,
    counts: HashMap<K, usize>,
    sum_count_log_count: f64,
}

impl<K: Copy + Eq + Hash> DiversityWindow<K> {
    fn new(capacity: usize) -> Self {
        // A zero window would hold no draw at all and score `None` forever, so clamp instead.
        let capacity = capacity.max(1);
        Self {
            capacity,
            // Grown by the draws themselves: reserving the configured window up front would commit
            // its whole length on the first draw, for a node that may never reach it.
            drawn: VecDeque::new(),
            counts: HashMap::default(),
            sum_count_log_count: 0.0,
        }
    }

    /// Records one draw, dropping the oldest beyond the window size.
    ///
    /// Returns the draw that fell out of the window, so a caller keeping a running total can
    /// retract it without walking the window.
    fn record(&mut self, value: K) -> Option<K> {
        let evicted = if self.drawn.len() == self.capacity {
            self.drawn.pop_front()
        } else {
            None
        };
        if let Some(oldest) = evicted {
            let count = self.counts.get_mut(&oldest).copied().unwrap_or(0);
            self.sum_count_log_count += count_log_count_delta(count, count.saturating_sub(1));
            match count {
                0 | 1 => {
                    self.counts.remove(&oldest);
                }
                _ => {
                    self.counts.insert(oldest, count - 1);
                }
            }
        }
        let count = self.counts.get(&value).copied().unwrap_or(0);
        self.sum_count_log_count += count_log_count_delta(count, count + 1);
        self.counts.insert(value, count + 1);
        self.drawn.push_back(value);
        evicted
    }

    /// Effective number of distinct values in the window; `None` while it holds no draw.
    fn score(&self) -> Option<f64> {
        effective_size(self.drawn.len(), self.counts.len(), self.sum_count_log_count)
    }
}

/// The node-wide total of one dimension's draws.
///
/// Only the gauges read a pooled score, so a build without telemetry does no pooled work at all:
/// the windows still feed the per-session WARN, but keeping a second set of counts in step would
/// be a cost with no reader.
#[cfg(any(test, feature = "telemetry"))]
#[derive(Debug)]
struct Pooled<K> {
    counts: HashMap<K, usize>,
    drawn: usize,
    sum_count_log_count: f64,
}

#[cfg(any(test, feature = "telemetry"))]
impl<K: Copy + Eq + Hash> Default for Pooled<K> {
    fn default() -> Self {
        Self {
            counts: HashMap::default(),
            drawn: 0,
            sum_count_log_count: 0.0,
        }
    }
}

#[cfg(any(test, feature = "telemetry"))]
impl<K: Copy + Eq + Hash> Pooled<K> {
    fn add(&mut self, value: K) {
        let count = self.counts.get(&value).copied().unwrap_or(0);
        self.sum_count_log_count += count_log_count_delta(count, count + 1);
        self.counts.insert(value, count + 1);
        self.drawn += 1;
    }

    /// Removes `drawn` draws of `value`, dropping the entry once no window contributes to it.
    fn retract(&mut self, value: K, drawn: usize) {
        self.drawn = self.drawn.saturating_sub(drawn);
        let count = self.counts.get(&value).copied().unwrap_or(0);
        let remaining = count.saturating_sub(drawn);
        self.sum_count_log_count += count_log_count_delta(count, remaining);
        if remaining == 0 {
            self.counts.remove(&value);
        } else {
            self.counts.insert(value, remaining);
        }
    }

    fn score(&self) -> Option<f64> {
        effective_size(self.drawn, self.counts.len(), self.sum_count_log_count)
    }

    fn draws(&self) -> usize {
        self.drawn
    }
}

#[derive(Debug)]
struct SessionWindows {
    forward_first_hop: DiversityWindow<OffchainPublicKey>,
    return_first_hop: DiversityWindow<OffchainPublicKey>,
    route: DiversityWindow<u64>,
    /// Draws each dimension has ever taken for this session, which the warm-up reads.
    ///
    /// Cumulative on purpose: the retained count never exceeds the window, so a window shorter
    /// than the warm-up would otherwise keep the score from ever being judged.
    forward_first_hop_drawn: usize,
    return_first_hop_drawn: usize,
    route_drawn: usize,
    /// Tracker tick of the last recorded draw, for least-recently-updated eviction.
    last_drawn: u64,
}

/// Records `values` into one window, keeping `pooled` in step with what the window retains.
fn record_batch<K: Copy + Eq + Hash>(
    window: &mut DiversityWindow<K>,
    pooled: &mut Pooled<K>,
    values: impl Iterator<Item = K>,
) {
    for value in values {
        if let Some(evicted) = window.record(value) {
            pooled.retract(evicted, 1);
        }
        pooled.add(value);
    }
}

/// Per-session windows plus the pooled score over all of them.
#[derive(Debug)]
pub(crate) struct DiversityTracker {
    window: usize,
    threshold: f64,
    ticks: u64,
    sessions: HashMap<DiversityKey, SessionWindows>,
    pooled_forward_first_hop: Pooled<OffchainPublicKey>,
    pooled_return_first_hop: Pooled<OffchainPublicKey>,
    pooled_route: Pooled<u64>,
    /// Sessions currently at or below `threshold` in one dimension, so the WARN fires on the
    /// transition into that state and the INFO once on leaving it, not on every draw.
    breached: HashSet<(Dimension, DiversityKey)>,
}

impl DiversityTracker {
    /// `window` is the number of draws each session's score covers, `threshold` the score at or
    /// below which a first-hop dimension is logged as concentrated (`0.0` disables the logging).
    pub(crate) fn new(window: usize, threshold: f64) -> Self {
        Self {
            window: window.max(1),
            threshold,
            ticks: 0,
            sessions: HashMap::default(),
            pooled_forward_first_hop: Pooled::default(),
            pooled_return_first_hop: Pooled::default(),
            pooled_route: Pooled::default(),
            breached: HashSet::new(),
        }
    }

    /// Records the forward path of one packet and the routes its SURBs take.
    ///
    /// `routes` holds one entry per SURB: a hash of the forward path together with that SURB's
    /// return path. Each dimension's threshold is evaluated once the whole batch is in, so a batch
    /// that dips a score for a moment does not log a WARN immediately followed by an INFO.
    pub(crate) fn record_forward_first_hop(&mut self, key: DiversityKey, first_hop: OffchainPublicKey) {
        self.tick_and_evict(key);
        {
            let session = self
                .sessions
                .get_mut(&key)
                .expect("the session was created by tick_and_evict");
            record_batch(
                &mut session.forward_first_hop,
                &mut self.pooled_forward_first_hop,
                std::iter::once(first_hop),
            );
            session.forward_first_hop_drawn += 1;
        }
        self.apply_threshold(key, Dimension::ForwardFirstHop);
    }

    /// Records the routes one packet's SURBs took, under the key of the return leg that chose them.
    ///
    /// Takes the routes as an iterator, so the packet path does not build a vector per packet.
    #[cfg(any(test, feature = "telemetry"))]
    pub(crate) fn record_routes(&mut self, key: DiversityKey, routes: impl Iterator<Item = u64> + Clone) {
        let drawn = routes.clone().count();
        if drawn == 0 {
            return;
        }
        self.tick_and_evict(key);
        {
            let session = self
                .sessions
                .get_mut(&key)
                .expect("the session was created by tick_and_evict");
            record_batch(&mut session.route, &mut self.pooled_route, routes);
            session.route_drawn += drawn;
        }
        self.apply_threshold(key, Dimension::Route);
    }

    /// Records the first relayers of every return path drawn for `key` in one batch.
    ///
    /// Takes the draws as an iterator, so the packet path does not build a vector per packet.
    pub(crate) fn record_returns(
        &mut self,
        key: DiversityKey,
        first_hops: impl Iterator<Item = OffchainPublicKey> + Clone,
    ) {
        let drawn = first_hops.clone().count();
        if drawn == 0 {
            return;
        }
        self.tick_and_evict(key);
        {
            let session = self
                .sessions
                .get_mut(&key)
                .expect("the session was created by tick_and_evict");
            record_batch(
                &mut session.return_first_hop,
                &mut self.pooled_return_first_hop,
                first_hops,
            );
            session.return_first_hop_drawn += drawn;
        }
        self.apply_threshold(key, Dimension::ReturnFirstHop);
    }

    /// Pooled score of every dimension; `None` for a dimension nothing was drawn in.
    #[cfg_attr(not(any(test, feature = "telemetry")), allow(dead_code))]
    pub(crate) fn scores(&self) -> Scores {
        Scores {
            forward_first_hop: self.pooled_forward_first_hop.score(),
            return_first_hop: self.pooled_return_first_hop.score(),
            route: self.pooled_route.score(),
            forward_first_hop_draws: self.pooled_forward_first_hop.draws(),
            return_first_hop_draws: self.pooled_return_first_hop.draws(),
            route_draws: self.pooled_route.draws(),
        }
    }

    /// One session's score in one dimension; `None` if it has no window or no draw in it.
    #[cfg(test)]
    pub(crate) fn session_score(&self, key: &DiversityKey, dimension: Dimension) -> Option<f64> {
        self.score_of(key, dimension)
    }

    /// Sessions currently tracked, for tests asserting the bound.
    #[cfg(test)]
    fn tracked_sessions(&self) -> usize {
        self.sessions.len()
    }

    /// Marks `key` as just drawn in, creating its windows and evicting the least recent session
    /// when the bound is reached.
    fn tick_and_evict(&mut self, key: DiversityKey) {
        self.ticks += 1;
        let tick = self.ticks;

        if !self.sessions.contains_key(&key) && self.sessions.len() >= MAX_TRACKED_SESSIONS {
            self.evict_least_recently_updated();
        }
        let window_size = self.window;
        let session = self.sessions.entry(key).or_insert_with(|| SessionWindows {
            forward_first_hop: DiversityWindow::new(window_size),
            return_first_hop: DiversityWindow::new(window_size),
            route: DiversityWindow::new(window_size),
            forward_first_hop_drawn: 0,
            return_first_hop_drawn: 0,
            route_drawn: 0,
            last_drawn: tick,
        });
        session.last_drawn = tick;
    }

    /// Draws `key` has ever taken in `dimension`, which the warm-up requires before a WARN.
    fn drawn_total_of(&self, key: &DiversityKey, dimension: Dimension) -> usize {
        let Some(session) = self.sessions.get(key) else {
            return 0;
        };
        match dimension {
            Dimension::ForwardFirstHop => session.forward_first_hop_drawn,
            Dimension::ReturnFirstHop => session.return_first_hop_drawn,
            Dimension::Route => session.route_drawn,
        }
    }

    fn score_of(&self, key: &DiversityKey, dimension: Dimension) -> Option<f64> {
        let session = self.sessions.get(key)?;
        match dimension {
            Dimension::ForwardFirstHop => session.forward_first_hop.score(),
            Dimension::ReturnFirstHop => session.return_first_hop.score(),
            Dimension::Route => session.route.score(),
        }
    }

    fn evict_least_recently_updated(&mut self) {
        let Some(oldest) = self
            .sessions
            .iter()
            .min_by_key(|(_, session)| session.last_drawn)
            .map(|(key, _)| *key)
        else {
            return;
        };
        let Some(session) = self.sessions.remove(&oldest) else {
            return;
        };
        // The evicted session's draws leave the node-wide total with it: a relayer it alone used
        // must not keep flattering a pooled score. Counted down per value, not per draw: a window
        // can hold one relayer thousands of times.
        for (value, count) in session.forward_first_hop.counts.iter() {
            self.pooled_forward_first_hop.retract(*value, *count);
        }
        for (value, count) in session.return_first_hop.counts.iter() {
            self.pooled_return_first_hop.retract(*value, *count);
        }
        for (value, count) in session.route.counts.iter() {
            self.pooled_route.retract(*value, *count);
        }
        // A session that comes back must re-arm its WARN rather than be treated as still breached.
        self.breached.retain(|(_, key)| key != &oldest);
    }

    /// Logs one WARN when `key` first reaches the threshold in `dimension`, one INFO when it leaves.
    ///
    /// Only the first-hop dimensions are checked: they name the relays that see the pair, where a
    /// single relayer is the case that needs attention.
    fn apply_threshold(&mut self, key: DiversityKey, dimension: Dimension) {
        // `0.0` is the documented way to turn the check off: no score can fall to or below it.
        if self.threshold <= 0.0 || dimension == Dimension::Route {
            return;
        }
        let Some(score) = self.score_of(&key, dimension) else {
            return;
        };
        if self.drawn_total_of(&key, dimension) < MIN_DRAWS_FOR_THRESHOLD {
            return;
        }
        let breached = (dimension, key);
        if score <= self.threshold {
            if self.breached.insert(breached) {
                tracing::warn!(
                    destination = %key.0,
                    hops = key.1,
                    dimension = ?dimension,
                    score,
                    threshold = self.threshold,
                    "path relayer diversity at or below the configured minimum",
                );
            }
        } else if self.breached.remove(&breached) {
            tracing::info!(
                destination = %key.0,
                hops = key.1,
                dimension = ?dimension,
                score,
                threshold = self.threshold,
                "path relayer diversity recovered",
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Context as _;
    use hopr_api::types::crypto::prelude::{Keypair, OffchainKeypair};

    use super::*;

    fn relayer(byte: u8) -> OffchainPublicKey {
        *OffchainKeypair::from_secret(&[byte; 32])
            .expect("valid secret")
            .public()
    }

    fn destination() -> OffchainPublicKey {
        relayer(0xff)
    }

    fn key() -> DiversityKey {
        (destination(), 1)
    }

    fn assert_close(expected: f64, actual: f64) {
        assert!((expected - actual).abs() < 1e-9, "expected {expected}, got {actual}");
    }

    /// Records `count` forward first-hop draws that alternate over `hops`.
    fn draw_forward(tracker: &mut DiversityTracker, hops: &[OffchainPublicKey], count: usize) {
        for i in 0..count {
            tracker.record_forward_first_hop(key(), hops[i % hops.len()]);
        }
    }

    /// Records `count` routes that alternate over `routes`, as one SURB each.
    fn draw_routes(tracker: &mut DiversityTracker, routes: &[u64], count: usize) {
        for i in 0..count {
            tracker.record_routes(key(), [routes[i % routes.len()]].into_iter());
        }
    }

    /// The score of an even spread is the number of distinct values it covers: that is what makes
    /// the gauge readable as an anonymity-set size rather than as raw entropy.
    #[test]
    fn uniform_draws_score_the_number_of_distinct_relayers() -> anyhow::Result<()> {
        let mut tracker = DiversityTracker::new(100, 0.0);
        draw_forward(&mut tracker, &[relayer(1), relayer(2), relayer(3), relayer(4)], 100);

        assert_close(
            4.0,
            tracker
                .session_score(&key(), Dimension::ForwardFirstHop)
                .context("forward score of an even spread over four relayers")?,
        );
        anyhow::Ok(())
    }

    /// Every draw through one relayer is the fully concentrated case, and scores exactly 1.0 so the
    /// default threshold of 1.0 catches it.
    #[test]
    fn collapsed_first_hop_scores_one() -> anyhow::Result<()> {
        let mut tracker = DiversityTracker::new(100, 0.0);
        draw_forward(&mut tracker, &[relayer(1)], 100);
        tracker.record_returns(key(), [relayer(2); 10].into_iter());

        assert_close(
            1.0,
            tracker
                .session_score(&key(), Dimension::ForwardFirstHop)
                .context("forward score of a single-relayer window")?,
        );
        assert_close(
            1.0,
            tracker
                .session_score(&key(), Dimension::ReturnFirstHop)
                .context("return score of a single-relayer window")?,
        );
        anyhow::Ok(())
    }

    /// The two directions are scored apart, which is the point of measuring them separately: a
    /// session can keep its forward legs varied while every return path leaves through one relayer.
    #[test]
    fn the_two_directions_are_scored_separately() -> anyhow::Result<()> {
        let mut tracker = DiversityTracker::new(100, 0.0);
        for i in 0..20 {
            tracker.record_forward_first_hop(key(), relayer(1 + (i % 2) as u8));
            tracker.record_returns(key(), [relayer(3)].into_iter());
        }

        assert_close(
            2.0,
            tracker
                .session_score(&key(), Dimension::ForwardFirstHop)
                .context("forward score over two relayers")?,
        );
        assert_close(
            1.0,
            tracker
                .session_score(&key(), Dimension::ReturnFirstHop)
                .context("return score over one relayer")?,
        );

        // Pooled over both directions the node still reads 2.0, one relayer per direction.
        let scores = tracker.scores();
        assert_close(2.0, scores.forward_first_hop.context("pooled forward score")?);
        assert_close(1.0, scores.return_first_hop.context("pooled return score")?);
        anyhow::Ok(())
    }

    /// The window is a window: draws that have fallen out of it must stop counting, or a relayer
    /// this node has not used in hundreds of draws would keep flattering the score.
    #[test]
    fn window_eviction_drops_the_oldest_draws() -> anyhow::Result<()> {
        let mut tracker = DiversityTracker::new(4, 0.0);
        draw_forward(&mut tracker, &[relayer(1)], 4);
        tracker.record_forward_first_hop(key(), relayer(2));
        // Three of relayer 1 and one of relayer 2: H = -(0.75 ln 0.75 + 0.25 ln 0.25).
        assert_close(
            1.754_765_350_603_323_2,
            tracker
                .session_score(&key(), Dimension::ForwardFirstHop)
                .context("score of a partly turned-over window")?,
        );

        draw_forward(&mut tracker, &[relayer(2)], 4);
        assert_close(
            1.0,
            tracker
                .session_score(&key(), Dimension::ForwardFirstHop)
                .context("score after the window turned over")?,
        );
        anyhow::Ok(())
    }

    /// Repeated routes are one route. The route dimension reads the pair a session actually uses,
    /// so a stable forward path paired with a stable return path scores 1.0 even when both are long.
    #[test]
    fn repeated_routes_score_one_and_distinct_routes_score_their_count() -> anyhow::Result<()> {
        let mut tracker = DiversityTracker::new(100, 0.0);
        draw_routes(&mut tracker, &[7], 10);
        assert_close(
            1.0,
            tracker
                .session_score(&key(), Dimension::Route)
                .context("score of one repeated route")?,
        );

        for route in [8, 9] {
            draw_routes(&mut tracker, &[route], 10);
        }
        assert_close(
            3.0,
            tracker
                .session_score(&key(), Dimension::Route)
                .context("score of three evenly used routes")?,
        );
        anyhow::Ok(())
    }

    /// A window holding one relayer must read exactly 1.0, not 1.0000000000000009: the WARN and
    /// the alert compare against 1.0, and a collapse they miss is the case they exist for.
    #[test]
    fn a_single_relayer_reads_exactly_one() -> anyhow::Result<()> {
        let mut tracker = DiversityTracker::new(200, 1.0);
        // Six draws through one relayer, then enough through another to evict every one of them.
        draw_forward(&mut tracker, &[relayer(1)], 6);
        draw_forward(&mut tracker, &[relayer(2)], 400);

        assert_eq!(
            1.0,
            tracker
                .session_score(&key(), Dimension::ForwardFirstHop)
                .context("session score of a single relayer")?,
        );
        assert_eq!(
            1.0,
            tracker
                .scores()
                .forward_first_hop
                .context("pooled score of a single relayer")?,
        );
        anyhow::Ok(())
    }

    /// The warm-up counts draws, not retained draws: a window shorter than the warm-up would
    /// otherwise never let its score be judged.
    #[tokio::test]
    #[serial_test::serial(routing_logs)]
    async fn a_window_shorter_than_the_warm_up_still_warns() {
        let logs = crate::tests::support::Logs::default();
        let _subscriber = tracing::subscriber::set_default(logs.subscriber(tracing::Level::TRACE));
        let _other_dispatch = tracing::Dispatch::new(tracing::subscriber::NoSubscriber::default());
        let mut tracker = DiversityTracker::new(5, 1.0);

        draw_forward(&mut tracker, &[relayer(1)], MIN_DRAWS_FOR_THRESHOLD * 2);

        assert_eq!(
            1,
            logs.records("path relayer diversity at or below the configured minimum")
                .len(),
            "a five-draw window must still judge a score once enough draws have been taken"
        );
    }

    /// The published counts follow the pool, so an alert that requires data stops firing once the
    /// draws it described are gone.
    #[test]
    fn the_published_counts_follow_the_pooled_draws() -> anyhow::Result<()> {
        let mut tracker = DiversityTracker::new(10, 0.0);
        tracker.record_returns(key(), [relayer(1), relayer(2)].into_iter());
        let scores = tracker.scores();
        assert_eq!(2, scores.return_first_hop_draws);
        assert_eq!(0, scores.route_draws);

        // Evict the only session that ever drew a return path.
        for i in 0..(MAX_TRACKED_SESSIONS as u32) {
            let key: DiversityKey = (relayer((i % 200) as u8 + 2), 1 + i / 200);
            tracker.record_forward_first_hop(key, relayer(9));
        }
        let scores = tracker.scores();
        assert_eq!(None, scores.return_first_hop, "the pooled score must be empty");
        assert_eq!(0, scores.return_first_hop_draws, "and so must its draw count");
        assert_eq!(
            MAX_TRACKED_SESSIONS, scores.forward_first_hop_draws,
            "the sessions that replaced it drew forward hops of their own"
        );
        anyhow::Ok(())
    }

    /// An empty window has no score. Reporting 0.0 or 1.0 there would put a fabricated reading on
    /// the gauges before the node has resolved a single path.
    #[test]
    fn an_empty_tracker_has_no_scores() {
        let tracker = DiversityTracker::new(10, 1.0);
        let scores = tracker.scores();
        assert_eq!(None, scores.forward_first_hop);
        assert_eq!(None, scores.return_first_hop);
        assert_eq!(None, scores.route);
        assert_eq!(None, tracker.session_score(&key(), Dimension::ForwardFirstHop));
    }

    /// Sessions are scored on their own draws, so one session riding a single relayer cannot be
    /// hidden by another spreading its own.
    #[test]
    fn sessions_keep_independent_windows() -> anyhow::Result<()> {
        let mut tracker = DiversityTracker::new(10, 0.0);
        let other: DiversityKey = (relayer(0xfe), 2);
        for i in 0..10 {
            tracker.record_forward_first_hop(key(), relayer(1 + (i % 2) as u8));
            tracker.record_forward_first_hop(other, relayer(3));
        }

        assert_close(
            2.0,
            tracker
                .session_score(&key(), Dimension::ForwardFirstHop)
                .context("spread session's own score")?,
        );
        assert_close(
            1.0,
            tracker
                .session_score(&other, Dimension::ForwardFirstHop)
                .context("concentrated session's own score")?,
        );
        anyhow::Ok(())
    }

    /// The tracker is bounded: without eviction a node with a churning set of destinations would
    /// hold a window per destination forever.
    #[test]
    fn tracker_evicts_the_least_recently_updated_session_beyond_the_bound() {
        let mut tracker = DiversityTracker::new(10, 0.0);
        let first: DiversityKey = (relayer(1), 1);
        tracker.record_forward_first_hop(first, relayer(1));

        for i in 0..MAX_TRACKED_SESSIONS {
            // A distinct destination per session, none of them the one recorded first.
            let key: DiversityKey = (relayer((i % 200) as u8 + 2), (i / 200) as u32);
            tracker.record_forward_first_hop(key, relayer((i % 3) as u8));
        }

        assert_eq!(MAX_TRACKED_SESSIONS, tracker.tracked_sessions());
        assert_eq!(
            None,
            tracker.session_score(&first, Dimension::ForwardFirstHop),
            "the least recently updated session must be the one evicted"
        );
    }

    /// A session's draws leave the node-wide totals when the session is evicted, in every
    /// dimension. Keeping them would let relayers and routes this node no longer uses hold the
    /// pooled scores up.
    #[test]
    fn evicting_a_session_drops_its_draws_from_the_pooled_scores() -> anyhow::Result<()> {
        let mut tracker = DiversityTracker::new(10, 0.0);
        let own: DiversityKey = (relayer(1), 1);
        for _ in 0..10 {
            tracker.record_forward_first_hop(own, relayer(1));
            tracker.record_routes(own, [1].into_iter());
        }
        let before = tracker.scores();
        assert_close(1.0, before.forward_first_hop.context("pooled forward score")?);
        assert_close(1.0, before.route.context("pooled route score")?);

        // Fill the tracker with sessions that use only relayer 9 and route 9.
        for i in 0..(MAX_TRACKED_SESSIONS as u32) {
            let key: DiversityKey = (relayer((i % 200) as u8 + 2), 1 + i / 200);
            tracker.record_forward_first_hop(key, relayer(9));
            tracker.record_routes(key, [9].into_iter());
        }

        assert_eq!(
            None,
            tracker.session_score(&own, Dimension::ForwardFirstHop),
            "the first session must be the one evicted"
        );
        // Only relayer 9 and route 9 remain in the pooled totals, so both scores must be 1.0.
        let after = tracker.scores();
        assert_close(1.0, after.forward_first_hop.context("pooled forward score")?);
        assert_close(1.0, after.route.context("pooled route score")?);
        anyhow::Ok(())
    }

    /// A batch can evict its own earlier draws. The pooled total has to follow the window through
    /// that in draw order, or it keeps a route the window has already dropped and reports a spread
    /// that does not exist: with a window of 1, drawing `[A, B]` is one route, not two.
    #[test]
    fn a_batch_that_outgrows_the_window_leaves_no_trace_in_the_pooled_score() -> anyhow::Result<()> {
        let mut tracker = DiversityTracker::new(1, 0.0);

        tracker.record_routes(key(), [1, 2].into_iter());
        assert_close(
            1.0,
            tracker
                .session_score(&key(), Dimension::Route)
                .context("route score after drawing [A, B]")?,
        );
        assert_close(
            1.0,
            tracker
                .scores()
                .route
                .context("pooled route score after drawing [A, B]")?,
        );

        // B is now replaced by C: still one route in the window.
        tracker.record_routes(key(), [3].into_iter());
        assert_close(
            1.0,
            tracker
                .session_score(&key(), Dimension::Route)
                .context("route score after replacing B with C")?,
        );
        assert_close(
            1.0,
            tracker.scores().route.context("pooled route score after replacing B")?,
        );
        anyhow::Ok(())
    }

    /// One WARN on the transition into the concentrated state, one INFO on leaving it, and nothing
    /// in between: a per-draw log would drown the node in warnings. Each direction is tracked
    /// separately, and the record names the dimension that breached.
    #[tokio::test]
    #[serial_test::serial(routing_logs)]
    async fn the_threshold_logs_only_on_the_transitions() -> anyhow::Result<()> {
        let logs = crate::tests::support::Logs::default();
        let _subscriber = tracing::subscriber::set_default(logs.subscriber(tracing::Level::TRACE));
        // Keeps callsite interest out of the cached-`never` state that a lone scoped dispatcher
        // leaves behind for callsites first reached on a runtime worker thread.
        let _other_dispatch = tracing::Dispatch::new(tracing::subscriber::NoSubscriber::default());
        let mut tracker = DiversityTracker::new(100, 1.5);

        // Ten forward draws through one relayer put the score at 1.0, below the 1.5 threshold.
        draw_forward(&mut tracker, &[relayer(1)], 10);
        let breach = logs.records("path relayer diversity at or below the configured minimum");
        assert_eq!(1, breach.len(), "the breach must be logged once");
        assert_eq!("ForwardFirstHop", breach[0].fields["dimension"]);

        // Ten more draws of the same relayer keep it breached: still one WARN.
        draw_forward(&mut tracker, &[relayer(1)], 10);
        assert_eq!(
            1,
            logs.records("path relayer diversity at or below the configured minimum")
                .len(),
            "a repeated draw in the breached state must not re-log"
        );

        // A return draw through a different single relayer breaches that dimension on its own,
        // once it has drawn enough for the score to mean anything.
        tracker.record_returns(key(), [relayer(2); 10].into_iter());
        let breach = logs.records("path relayer diversity at or below the configured minimum");
        assert_eq!(2, breach.len(), "the return dimension must warn on its own breach");
        assert_eq!("ReturnFirstHop", breach[1].fields["dimension"]);

        // Draws through a second relayer lift the forward score above the threshold, monotonically:
        // exactly one INFO. An alternating pattern would cross back and forth and log each crossing.
        draw_forward(&mut tracker, &[relayer(3)], 40);
        assert!(
            tracker
                .session_score(&key(), Dimension::ForwardFirstHop)
                .context("forward score after spreading the draws")?
                > 1.5
        );
        assert_eq!(
            1,
            logs.records("path relayer diversity recovered").len(),
            "only the recovered dimension may log"
        );
        anyhow::Ok(())
    }

    /// The first draws of a session always score 1.0, so the check must stay quiet until a window
    /// says something: otherwise every session that starts, and every one that returns after
    /// eviction, logs a WARN and an INFO about a score that could not have been anything else.
    #[tokio::test]
    #[serial_test::serial(routing_logs)]
    async fn the_first_draws_of_a_session_are_not_logged_as_concentrated() {
        let logs = crate::tests::support::Logs::default();
        let _subscriber = tracing::subscriber::set_default(logs.subscriber(tracing::Level::TRACE));
        let _other_dispatch = tracing::Dispatch::new(tracing::subscriber::NoSubscriber::default());
        let mut tracker = DiversityTracker::new(100, 1.0);

        draw_forward(&mut tracker, &[relayer(1)], MIN_DRAWS_FOR_THRESHOLD - 1);
        tracker.record_returns(key(), [relayer(2); MIN_DRAWS_FOR_THRESHOLD - 1].into_iter());

        assert!(
            logs.records("path relayer diversity at or below the configured minimum")
                .is_empty(),
            "a window below the draw minimum must not log"
        );
    }

    /// A zero threshold is the documented switch for the check, not a threshold nothing can miss.
    #[tokio::test]
    #[serial_test::serial(routing_logs)]
    async fn a_zero_threshold_logs_nothing() {
        let logs = crate::tests::support::Logs::default();
        let _subscriber = tracing::subscriber::set_default(logs.subscriber(tracing::Level::TRACE));
        let _other_dispatch = tracing::Dispatch::new(tracing::subscriber::NoSubscriber::default());
        let mut tracker = DiversityTracker::new(100, 0.0);

        draw_forward(&mut tracker, &[relayer(1)], 10);
        tracker.record_returns(key(), [relayer(2)].into_iter());

        assert!(
            logs.records("path relayer diversity at or below the configured minimum")
                .is_empty()
        );
        assert!(logs.records("path relayer diversity recovered").is_empty());
    }
}
