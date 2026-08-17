//! Sequences the response to a return path that has gone silent.
//!
//! Two mechanisms react to the same evidence and, left unsequenced, work against each other. The
//! planner learns to route around the relay that stopped delivering, while the SURB balancer floods
//! the counterparty to refill a buffer it believes is empty. Run in the wrong order the flood mints
//! its SURBs onto the very route the planner is in the middle of abandoning -- measured at ~38 000
//! SURBs in 14 s against a 15 000-entry ring buffer, which is 2.5x the counterparty's whole store,
//! so everything older is evicted and a LIFO reader reaches preferentially for the poisoned ones.
//!
//! The rule this module enforces is therefore: **re-plan first, refill only if re-planning actually
//! moved traffic.** A re-plan that moves nothing means the silent relay sits on every remaining
//! candidate -- return-path diversity caps at `HoprPacket::PAYLOAD_SIZE / HoprSurb::SIZE` = 2, so
//! this is an ordinary case, not a corner -- and refilling then only buys more SURBs bound for the
//! same dead route.

use std::{
    collections::HashMap,
    future::Future,
    hash::Hash,
    time::{Duration, Instant},
};

/// One action taken on behalf of one destination, recorded in the order it happened.
///
/// Exists so the ordering guarantee is observable: a caller (or a test) can see that no refill was
/// ever issued for a destination before the re-plan that justified it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryStep<K> {
    /// The destination's cached return paths were rebuilt; `moved` entries came back re-weighted.
    Replanned {
        /// The destination whose paths were rebuilt.
        destination: K,
        /// How many cached entries the re-plan actually replaced.
        moved: usize,
    },
    /// The destination's Sessions were told to refill; `sessions` of them were marked.
    Refilled {
        /// The destination whose Sessions were marked.
        destination: K,
        /// How many Sessions routed there were marked.
        sessions: usize,
    },
}

/// Tracks which destinations already have an open recovery episode.
///
/// An episode opens the first tick a destination is reported silent and lapses after `grace`. While
/// it is open the destination is ignored, which bounds re-planning -- a full path rediscovery per
/// cached entry -- to once per grace window however long the silence lasts. A destination that is
/// still silent when the episode lapses simply opens a fresh one, so a re-plan that could not move
/// traffic the first time is retried later rather than abandoned.
pub struct ReturnPathEpisodes<K> {
    open: HashMap<K, Instant>,
    grace: Duration,
}

impl<K: Eq + Hash + Copy> ReturnPathEpisodes<K> {
    /// Creates a tracker whose episodes lapse after `grace`.
    pub fn new(grace: Duration) -> Self {
        Self {
            open: HashMap::new(),
            grace,
        }
    }

    /// Drives one flush tick over the destinations the detector reported silent.
    ///
    /// `replan` returns how many cached entries it re-weighted, and `refill` is invoked only when
    /// that count is non-zero. Both are injected so the sequencing can be exercised without a graph,
    /// a planner or a cluster.
    pub async fn tick<S, R, RFut, F, FFut>(&mut self, silent: S, mut replan: R, mut refill: F) -> Vec<RecoveryStep<K>>
    where
        S: IntoIterator<Item = K>,
        R: FnMut(K) -> RFut,
        RFut: Future<Output = usize>,
        F: FnMut(K) -> FFut,
        FFut: Future<Output = usize>,
    {
        let now = Instant::now();
        self.open.retain(|_, lapses_at| *lapses_at > now);

        let mut steps = Vec::new();
        for destination in silent {
            if self.open.contains_key(&destination) {
                continue;
            }
            self.open.insert(destination, now + self.grace);

            let moved = replan(destination).await;
            steps.push(RecoveryStep::Replanned { destination, moved });

            // Nothing moved: the silent relay is on every candidate that remains, so refilling
            // would only mint more SURBs onto the same route. Wait for the next episode instead.
            if moved > 0 {
                let sessions = refill(destination).await;
                steps.push(RecoveryStep::Refilled { destination, sessions });
            }
        }
        steps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Counts calls so a test can assert how often each side ran, not merely what it returned.
    #[derive(Default)]
    struct Calls {
        replans: std::cell::Cell<usize>,
        refills: std::cell::Cell<usize>,
    }

    /// Refilling before the re-plan is the defect this module exists to prevent: it is what put
    /// ~38 000 SURBs onto a route the planner was abandoning.
    #[tokio::test]
    async fn a_refill_should_never_precede_the_replan_that_justifies_it() {
        let mut episodes = ReturnPathEpisodes::new(Duration::from_secs(10));

        let steps = episodes.tick([1u32, 2u32], |_| async { 3 }, |_| async { 1 }).await;

        assert_eq!(steps.len(), 4, "each destination should re-plan and then refill");
        for destination in [1u32, 2u32] {
            let replanned = steps
                .iter()
                .position(|s| matches!(s, RecoveryStep::Replanned { destination: d, .. } if *d == destination))
                .expect("destination should have been re-planned");
            let refilled = steps
                .iter()
                .position(|s| matches!(s, RecoveryStep::Refilled { destination: d, .. } if *d == destination))
                .expect("destination should have been refilled");
            assert!(
                replanned < refilled,
                "destination {destination} refilled at step {refilled} before re-planning at {replanned}"
            );
        }
    }

    /// Re-planning rediscovers every path to the destination, so a destination that stays silent
    /// must not pay that cost on every flush tick.
    #[tokio::test]
    async fn an_open_episode_should_replan_once_however_long_the_silence_lasts() {
        let mut episodes = ReturnPathEpisodes::new(Duration::from_secs(10));
        let calls = Calls::default();

        for _ in 0..5 {
            episodes
                .tick(
                    [7u32],
                    |_| {
                        calls.replans.set(calls.replans.get() + 1);
                        async { 2 }
                    },
                    |_| {
                        calls.refills.set(calls.refills.get() + 1);
                        async { 1 }
                    },
                )
                .await;
        }

        assert_eq!(calls.replans.get(), 1, "one episode must mean one re-plan");
        assert_eq!(calls.refills.get(), 1, "one episode must mean one refill");
    }

    /// When the silent relay sits on every remaining candidate, re-planning cannot move traffic —
    /// and refilling then buys nothing but more SURBs bound for the same dead route.
    #[tokio::test]
    async fn a_replan_that_moves_nothing_should_not_refill() {
        let mut episodes = ReturnPathEpisodes::new(Duration::from_secs(10));
        let calls = Calls::default();

        let steps = episodes
            .tick(
                [7u32],
                |_| async { 0 },
                |_| {
                    calls.refills.set(calls.refills.get() + 1);
                    async { 1 }
                },
            )
            .await;

        assert_eq!(calls.refills.get(), 0, "a re-plan that moved nothing must not refill");
        assert_eq!(
            steps,
            vec![RecoveryStep::Replanned {
                destination: 7u32,
                moved: 0
            }],
            "the re-plan itself must still be recorded"
        );

        // Vacuity guard: the same fixture with a re-plan that *did* move traffic must refill,
        // otherwise this test would pass against a machine that never refills at all.
        let mut moved_episodes = ReturnPathEpisodes::new(Duration::from_secs(10));
        let moved_steps = moved_episodes.tick([7u32], |_| async { 1 }, |_| async { 4 }).await;
        assert!(
            moved_steps
                .iter()
                .any(|s| matches!(s, RecoveryStep::Refilled { sessions: 4, .. })),
            "a re-plan that moved traffic must refill: {moved_steps:?}"
        );
    }

    /// A destination that is still silent once its episode lapses has to be handled afresh — that
    /// retry is what lets a re-plan which could not move traffic the first time succeed later.
    #[tokio::test]
    async fn a_lapsed_episode_should_be_handled_afresh() {
        let mut episodes = ReturnPathEpisodes::new(Duration::from_millis(50));
        let calls = Calls::default();

        let bump = |_| {
            calls.replans.set(calls.replans.get() + 1);
            async { 2 }
        };

        episodes.tick([7u32], bump, |_| async { 1 }).await;
        assert_eq!(calls.replans.get(), 1, "the first tick opens an episode");

        tokio::time::sleep(Duration::from_millis(80)).await;

        episodes.tick([7u32], bump, |_| async { 1 }).await;
        assert_eq!(
            calls.replans.get(),
            2,
            "once the grace lapses the same destination must be handled afresh"
        );
    }
}
