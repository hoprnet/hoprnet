//! Anonymity-set diversity of the return paths a session draws.
//!
//! Return paths are drawn weighted-random by path value, so a session's SURBs concentrate on the
//! few best-scoring relayers even when the graph offers several. The planner already warns when the
//! *candidate pool* has collapsed to a single first relayer, but that says nothing about where the
//! draws actually go over time: a pool of ten can still serve one relayer for hours.
//!
//! This module measures that. It keeps a sliding window of the first relayer of every drawn return
//! path, per session, and scores the window as the effective number of distinct relayers,
//! `exp(H)` for the Shannon entropy `H` of the draw distribution. The score is 1.0 when every draw
//! went through one relayer and tends to the number of distinct relayers as the draws even out, so
//! an operator can alert on a score that stays near 1.

use std::collections::{HashMap, HashSet, VecDeque};

use hopr_api::OffchainPublicKey;

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

/// Effective number of distinct relayers of a draw distribution: `exp(H)` over the counts.
///
/// `None` when nothing was drawn: the score is undefined there, and a caller must not publish an
/// invented value.
fn effective_size<'a>(counts: impl Iterator<Item = &'a usize>, drawn: usize) -> Option<f64> {
    if drawn == 0 {
        return None;
    }
    let total = drawn as f64;
    let entropy = counts
        .filter(|count| **count > 0)
        .map(|count| {
            let share = *count as f64 / total;
            -share * share.ln()
        })
        .sum::<f64>();
    Some(entropy.exp())
}

/// The last `capacity` first relayers a session drew, with their counts kept alongside.
#[derive(Debug)]
pub(crate) struct DiversityWindow {
    capacity: usize,
    drawn: VecDeque<OffchainPublicKey>,
    counts: HashMap<OffchainPublicKey, usize>,
}

impl DiversityWindow {
    pub(crate) fn new(capacity: usize) -> Self {
        // A zero window would hold no draw at all and score `None` forever, so clamp instead.
        let capacity = capacity.max(1);
        Self {
            capacity,
            drawn: VecDeque::with_capacity(capacity),
            counts: HashMap::new(),
        }
    }

    /// Records one drawn first relayer, dropping the oldest draw beyond the window size.
    ///
    /// Returns the draw that fell out of the window, so a caller keeping a running total can
    /// retract it without walking the window.
    pub(crate) fn record(&mut self, relayer: OffchainPublicKey) -> Option<OffchainPublicKey> {
        let evicted = if self.drawn.len() == self.capacity {
            self.drawn.pop_front()
        } else {
            None
        };
        if let Some(oldest) = evicted {
            match self.counts.get_mut(&oldest) {
                Some(count) if *count > 1 => *count -= 1,
                Some(_) => {
                    self.counts.remove(&oldest);
                }
                None => {}
            }
        }
        *self.counts.entry(relayer).or_insert(0) += 1;
        self.drawn.push_back(relayer);
        evicted
    }

    /// Effective number of distinct relayers in the window; `None` while it holds no draw.
    pub(crate) fn score(&self) -> Option<f64> {
        effective_size(self.counts.values(), self.drawn.len())
    }
}

#[derive(Debug)]
struct SessionWindow {
    window: DiversityWindow,
    /// Tracker tick of the last recorded draw, for least-recently-updated eviction.
    last_drawn: u64,
}

/// Per-session diversity windows plus the pooled score over all of them.
#[derive(Debug)]
pub(crate) struct DiversityTracker {
    window: usize,
    threshold: f64,
    ticks: u64,
    sessions: HashMap<DiversityKey, SessionWindow>,
    /// Running total of every window's counts, so the pooled score costs the number of distinct
    /// relayers rather than every draw: it is recomputed on each return-path draw, which is the
    /// packet path.
    pooled: HashMap<OffchainPublicKey, usize>,
    pooled_drawn: usize,
    /// Sessions currently below `threshold`, so the WARN fires on the transition into that state
    /// and the INFO once on leaving it, not on every draw.
    breached: HashSet<DiversityKey>,
}

impl DiversityTracker {
    /// `window` is the number of draws each session's score covers, `threshold` the score at or
    /// below which a session is logged as concentrated (`0.0` disables the logging).
    pub(crate) fn new(window: usize, threshold: f64) -> Self {
        Self {
            window: window.max(1),
            threshold,
            ticks: 0,
            sessions: HashMap::new(),
            pooled: HashMap::new(),
            pooled_drawn: 0,
            breached: HashSet::new(),
        }
    }

    /// Records every first relayer of one draw batch for `key`.
    ///
    /// A whole batch is recorded before the threshold is evaluated, so a draw that briefly dips the
    /// score and recovers within the same batch does not log a WARN immediately followed by an INFO.
    pub(crate) fn record(&mut self, key: DiversityKey, relayers: &[OffchainPublicKey]) {
        if relayers.is_empty() {
            return;
        }
        self.ticks += 1;
        let tick = self.ticks;

        if !self.sessions.contains_key(&key) && self.sessions.len() >= MAX_TRACKED_SESSIONS {
            self.evict_least_recently_updated();
        }
        let window_size = self.window;
        let session = self.sessions.entry(key).or_insert_with(|| SessionWindow {
            window: DiversityWindow::new(window_size),
            last_drawn: tick,
        });
        session.last_drawn = tick;
        let mut evicted = Vec::with_capacity(relayers.len());
        for relayer in relayers {
            evicted.extend(session.window.record(*relayer));
        }

        let score = session.window.score();
        for gone in evicted {
            self.retract(gone);
        }
        for relayer in relayers {
            *self.pooled.entry(*relayer).or_insert(0) += 1;
            self.pooled_drawn += 1;
        }
        if let Some(score) = score {
            self.apply_threshold(key, score);
        }
    }

    /// Diversity score of one session; `None` if it has no window or no draw in it.
    #[cfg(test)]
    pub(crate) fn session_score(&self, key: &DiversityKey) -> Option<f64> {
        self.sessions.get(key).and_then(|session| session.window.score())
    }

    /// Diversity score over the draws of every tracked session together.
    ///
    /// This is the node-wide figure the gauge reports: it reads 1.0 only when every draw of every
    /// session went through the same relayer, so it can look diverse even when each session
    /// concentrates all its draws on a single relayer.
    #[cfg_attr(not(any(test, feature = "telemetry")), allow(dead_code))]
    pub(crate) fn pooled_score(&self) -> Option<f64> {
        effective_size(self.pooled.values(), self.pooled_drawn)
    }

    /// Sessions currently tracked, for tests asserting the bound.
    #[cfg(test)]
    fn tracked_sessions(&self) -> usize {
        self.sessions.len()
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
        // must not keep flattering the pooled score.
        for (relayer, count) in session.window.counts.iter() {
            for _ in 0..*count {
                self.retract(*relayer);
            }
        }
        // A session that comes back must re-arm its WARN rather than be treated as still breached.
        self.breached.remove(&oldest);
    }

    /// Removes one draw of `relayer` from the node-wide total.
    fn retract(&mut self, relayer: OffchainPublicKey) {
        if let Some(count) = self.pooled.get_mut(&relayer) {
            *count -= 1;
            if *count == 0 {
                self.pooled.remove(&relayer);
            }
        }
        self.pooled_drawn = self.pooled_drawn.saturating_sub(1);
    }

    fn apply_threshold(&mut self, key: DiversityKey, score: f64) {
        // `0.0` is the documented way to turn the check off: no score can fall to or below it.
        if self.threshold <= 0.0 {
            return;
        }
        if score <= self.threshold {
            if self.breached.insert(key) {
                tracing::warn!(
                    destination = %key.0,
                    hops = key.1,
                    score,
                    threshold = self.threshold,
                    "return-path relayer diversity at or below the configured minimum",
                );
            }
        } else if self.breached.remove(&key) {
            tracing::info!(
                destination = %key.0,
                hops = key.1,
                score,
                threshold = self.threshold,
                "return-path relayer diversity recovered",
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

    /// The score of an even spread is the number of distinct relayers it covers: that is what makes
    /// the gauge readable as an anonymity-set size rather than as raw entropy.
    #[test]
    fn uniform_draws_score_the_number_of_distinct_relayers() {
        let mut tracker = DiversityTracker::new(100, 0.0);
        for i in 0..100 {
            tracker.record(key(), &[relayer(i % 4)]);
        }
        assert_close(4.0, tracker.session_score(&key()).expect("drawn"));
    }

    /// Every draw through one relayer is the fully concentrated case, and scores exactly 1.0 so the
    /// default threshold of 1.0 catches it.
    #[test]
    fn collapsed_window_scores_one() {
        let mut tracker = DiversityTracker::new(100, 0.0);
        for _ in 0..100 {
            tracker.record(key(), &[relayer(1)]);
        }
        assert_close(1.0, tracker.session_score(&key()).expect("drawn"));
    }

    /// The window is a window: draws that have fallen out of it must stop counting, or a relayer
    /// this node has not used in hundreds of draws would keep flattering the score.
    #[test]
    fn window_eviction_drops_the_oldest_draws() {
        let mut tracker = DiversityTracker::new(4, 0.0);
        for _ in 0..4 {
            tracker.record(key(), &[relayer(1)]);
        }
        tracker.record(key(), &[relayer(2)]);
        // Three of relayer 1 and one of relayer 2: H = -(0.75 ln 0.75 + 0.25 ln 0.25).
        assert_close(1.754_765_350_603_323_2, tracker.session_score(&key()).expect("drawn"));

        for _ in 0..4 {
            tracker.record(key(), &[relayer(2)]);
        }
        assert_close(1.0, tracker.session_score(&key()).expect("drawn"));
    }

    /// An empty window has no score. Reporting 0.0 or 1.0 there would put a fabricated reading on
    /// the gauge before the node has drawn a single return path.
    #[test]
    fn an_empty_window_has_no_score() {
        let tracker = DiversityTracker::new(10, 1.0);
        assert_eq!(None, tracker.session_score(&key()));
        assert_eq!(None, tracker.pooled_score());
    }

    /// Sessions are scored on their own draws, so one session riding a single relayer cannot be
    /// hidden by another spreading its own.
    #[test]
    fn sessions_keep_independent_windows() {
        let mut tracker = DiversityTracker::new(10, 0.0);
        let other: DiversityKey = (relayer(0xfe), 2);
        for i in 0..10 {
            tracker.record(key(), &[relayer(i % 2)]);
            tracker.record(other, &[relayer(3)]);
        }
        assert_close(2.0, tracker.session_score(&key()).expect("drawn"));
        assert_close(1.0, tracker.session_score(&other).expect("drawn"));

        // Pooled over both sessions: ten draws each through relayers 1 and 2, twenty through
        // relayer 3, so the node-wide picture is more spread than either session's own.
        let pooled = tracker.pooled_score().expect("drawn");
        assert!(pooled > 2.0, "pooled score {pooled} must cover every session's draws");
    }

    /// The tracker is bounded: without eviction a node with a churning set of destinations would
    /// hold a window per destination forever.
    #[test]
    fn tracker_evicts_the_least_recently_updated_session_beyond_the_bound() {
        let mut tracker = DiversityTracker::new(10, 0.0);
        let first: DiversityKey = (relayer(1), 1);
        tracker.record(first, &[relayer(1)]);

        for i in 0..MAX_TRACKED_SESSIONS {
            // A distinct destination per session, none of them the one recorded first.
            let key: DiversityKey = (relayer((i % 200) as u8 + 2), (i / 200) as u32);
            tracker.record(key, &[relayer((i % 3) as u8)]);
        }

        assert_eq!(MAX_TRACKED_SESSIONS, tracker.tracked_sessions());
        assert_eq!(
            None,
            tracker.session_score(&first),
            "the least recently updated session must be the one evicted"
        );
    }

    /// A session's draws leave the node-wide total when the session is evicted. Keeping them would
    /// let a relayer this node no longer uses hold the pooled score up.
    #[test]
    fn evicting_a_session_drops_its_draws_from_the_pooled_score() {
        let mut tracker = DiversityTracker::new(10, 0.0);
        let own: DiversityKey = (relayer(1), 1);
        for _ in 0..10 {
            tracker.record(own, &[relayer(1)]);
        }
        assert_close(1.0, tracker.pooled_score().expect("drawn"));

        for i in 0..(MAX_TRACKED_SESSIONS as u32) {
            let key: DiversityKey = (relayer((i % 200) as u8 + 2), 1 + i / 200);
            tracker.record(key, &[relayer(9)]);
        }
        assert_eq!(
            None,
            tracker.session_score(&own),
            "the first session must be the one evicted"
        );
        // Only relayer 9 remains in the pooled total, so the node-wide score must be 1.0.
        assert_close(1.0, tracker.pooled_score().expect("drawn"));
    }

    /// One WARN on the transition into the concentrated state, one INFO on leaving it, and nothing
    /// in between: a per-draw log would drown the node in warnings.
    #[tokio::test]
    #[serial_test::serial(routing_logs)]
    async fn the_threshold_logs_only_on_the_transitions() -> anyhow::Result<()> {
        let logs = crate::tests::support::Logs::default();
        let _subscriber = tracing::subscriber::set_default(logs.subscriber(tracing::Level::TRACE));
        // Keeps callsite interest out of the cached-`never` state that a lone scoped dispatcher
        // leaves behind for callsites first reached on a runtime worker thread.
        let _other_dispatch = tracing::Dispatch::new(tracing::subscriber::NoSubscriber::default());
        let mut tracker = DiversityTracker::new(100, 1.5);

        // Ten draws through one relayer put the score at 1.0, below the 1.5 threshold.
        for _ in 0..10 {
            tracker.record(key(), &[relayer(1)]);
        }
        assert_eq!(
            1,
            logs.records("return-path relayer diversity at or below the configured minimum")
                .len(),
            "the breach must be logged once"
        );

        // Ten more draws of the same relayer keep it breached: still one WARN.
        for _ in 0..10 {
            tracker.record(key(), &[relayer(1)]);
        }
        assert_eq!(
            1,
            logs.records("return-path relayer diversity at or below the configured minimum")
                .len(),
            "a repeated draw in the breached state must not re-log"
        );

        // Spreading the draws lifts the score above the threshold: exactly one recovery INFO.
        for _ in 0..40 {
            tracker.record(key(), &[relayer(2)]);
        }
        assert!(
            tracker.session_score(&key()).context("drawn")? > 1.5,
            "the spread draws must lift the score above the threshold"
        );
        assert_eq!(
            1,
            logs.records("return-path relayer diversity recovered").len(),
            "the recovery must be logged once"
        );
        anyhow::Ok(())
    }

    /// A zero threshold is the documented switch for the check, not a threshold nothing can miss.
    #[tokio::test]
    #[serial_test::serial(routing_logs)]
    async fn a_zero_threshold_logs_nothing() {
        let logs = crate::tests::support::Logs::default();
        let _subscriber = tracing::subscriber::set_default(logs.subscriber(tracing::Level::TRACE));
        let _other_dispatch = tracing::Dispatch::new(tracing::subscriber::NoSubscriber::default());
        let mut tracker = DiversityTracker::new(100, 0.0);

        for _ in 0..10 {
            tracker.record(key(), &[relayer(1)]);
        }
        assert!(
            logs.records("return-path relayer diversity at or below the configured minimum")
                .is_empty()
        );
        assert!(logs.records("return-path relayer diversity recovered").is_empty());
    }
}
