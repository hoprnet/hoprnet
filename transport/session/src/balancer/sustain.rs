//! Bounding SURB production while the return path is degraded.
//!
//! A Session that opted into `sustain_on_return_path_loss` keeps producing while the return path
//! is marked degraded, because the counterparty keeps spending SURBs on replies that never arrive.
//! Consumption is then invisible, so the balancer assumes an empty buffer and refills on top of
//! the last consumption it saw while replies still arrived.
//!
//! Unbounded, that is a feedback loop: the extra upstream delays everything else, the delay reads
//! as more lost replies, and the mark is renewed for as long as production keeps it going. In the
//! field it ran at the full keep-alive budget for as long as the marks kept coming.
//!
//! So while degraded, production is capped at the last healthy consumption plus a surplus, and the
//! surplus halves every [`SURPLUS_HALF_LIFE`] for as long as the degradation lasts:
//!
//! - It starts at what a paced refill would add on top of consumption. That surplus is what pushes SURBs over the
//!   re-planned return paths into the counterparty's buffer, where the oldest -- those on the dead path -- are used or
//!   evicted first.
//! - It never drops below the consumption itself. The counterparty keeps spending at about that rate on a dead path
//!   too, and producing less would drain it. That keeps the recovery the opt-in exists for.
//! - A new degraded window opening within [`REENTRY_WINDOW`] of the last continues the same decay rather than starting
//!   over, so a return path that flaps between degraded and recovered cannot hold production at the initial surplus.

use std::time::{Duration, Instant};

use crate::balancer::paced::IDLE_REFILL_HORIZON;

/// How long the surplus above consumption takes to halve while the return path stays degraded.
const SURPLUS_HALF_LIFE: Duration = Duration::from_secs(5);

/// How soon after a degraded window ends a new one counts as the same episode.
const REENTRY_WINDOW: Duration = Duration::from_secs(30);

/// Tracks degraded episodes and the production cap that applies during them.
#[derive(Clone, Debug, Default)]
pub(crate) struct SustainBudget {
    /// When the current episode began.
    episode_started: Option<Instant>,
    /// When the return path was last seen degraded.
    last_degraded: Option<Instant>,
}

impl SustainBudget {
    /// Advances the episode to `now`, and returns the production cap if the return path is
    /// `degraded`.
    ///
    /// `healthy_consumption` is the net consumption last measured while replies still arrived.
    /// `target` is the buffer target, which sizes the surplus for a Session that consumed next to
    /// nothing.
    pub(crate) fn update(
        &mut self,
        now: Instant,
        degraded: bool,
        healthy_consumption: f64,
        target: u64,
    ) -> Option<u64> {
        if !degraded {
            return None;
        }

        let continues = self
            .last_degraded
            .is_some_and(|last| now.saturating_duration_since(last) <= REENTRY_WINDOW);
        if !continues {
            self.episode_started = Some(now);
        }
        self.last_degraded = Some(now);

        let degraded_for = self
            .episode_started
            .map_or(Duration::ZERO, |started| now.saturating_duration_since(started));
        let decay = 0.5f64.powf(degraded_for.as_secs_f64() / SURPLUS_HALF_LIFE.as_secs_f64());

        let consumption = healthy_consumption.max(0.0);
        let initial_surplus = consumption.max(target as f64 / IDLE_REFILL_HORIZON.as_secs_f64());
        Some((consumption + initial_surplus * decay).round() as u64)
    }

    /// How long the current degraded episode has lasted at `now`, if one is under way.
    pub(crate) fn episode_duration(&self, now: Instant) -> Option<Duration> {
        let last = self.last_degraded?;
        (now.saturating_duration_since(last) <= REENTRY_WINDOW)
            .then(|| {
                self.episode_started
                    .map(|started| now.saturating_duration_since(started))
            })
            .flatten()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONSUMPTION: f64 = 800.0;
    const TARGET: u64 = 1_000;

    fn cap(budget: &mut SustainBudget, at: Instant) -> u64 {
        budget
            .update(at, true, CONSUMPTION, TARGET)
            .expect("a degraded return path has a cap")
    }

    #[test]
    fn a_healthy_return_path_should_not_be_capped() {
        let mut budget = SustainBudget::default();
        assert_eq!(None, budget.update(Instant::now(), false, CONSUMPTION, TARGET));
        assert_eq!(None, budget.episode_duration(Instant::now()));
    }

    /// The episode opens with the surplus a paced refill would add: consumption again.
    #[test]
    fn an_episode_should_open_at_twice_the_consumption() {
        let mut budget = SustainBudget::default();
        assert_eq!(1_600, cap(&mut budget, Instant::now()));
    }

    #[test]
    fn the_surplus_should_halve_every_half_life() {
        let mut budget = SustainBudget::default();
        let start = Instant::now();
        cap(&mut budget, start);

        assert_eq!(1_200, cap(&mut budget, start + SURPLUS_HALF_LIFE));
        assert_eq!(1_000, cap(&mut budget, start + SURPLUS_HALF_LIFE * 2));
    }

    /// The counterparty keeps spending on a dead path too; production below that would drain it.
    #[test]
    fn a_long_episode_should_never_cap_below_the_consumption() {
        let mut budget = SustainBudget::default();
        let start = Instant::now();
        for s in 0..=600 {
            assert!(cap(&mut budget, start + Duration::from_secs(s)) >= CONSUMPTION as u64);
        }
        assert_eq!(CONSUMPTION as u64, cap(&mut budget, start + Duration::from_secs(600)));
    }

    /// A Session that consumed next to nothing still gets a surplus to push fresh SURBs with.
    #[test]
    fn an_idle_session_should_start_from_the_idle_surplus() {
        let mut budget = SustainBudget::default();
        let expected = (TARGET as f64 / IDLE_REFILL_HORIZON.as_secs_f64()) as u64;
        assert_eq!(Some(expected), budget.update(Instant::now(), true, 0.0, TARGET));
    }

    /// The loop seen in the field: degraded, recovered for a few seconds, degraded again. Each
    /// window restarting at the full surplus would hold production up indefinitely.
    #[test]
    fn a_window_reopening_soon_after_should_continue_the_decay() {
        let mut budget = SustainBudget::default();
        let start = Instant::now();
        cap(&mut budget, start);
        cap(&mut budget, start + Duration::from_secs(10));

        // Recovered for four seconds, then degraded again.
        assert_eq!(
            None,
            budget.update(start + Duration::from_secs(12), false, CONSUMPTION, TARGET)
        );
        let reopened = cap(&mut budget, start + Duration::from_secs(14));

        let decayed = CONSUMPTION + CONSUMPTION * 0.5f64.powf(14.0 / SURPLUS_HALF_LIFE.as_secs_f64());
        assert_eq!(decayed.round() as u64, reopened);
        assert_eq!(
            Some(Duration::from_secs(14)),
            budget.episode_duration(start + Duration::from_secs(14))
        );
    }

    #[test]
    fn a_window_opening_long_after_should_start_a_new_episode() {
        let mut budget = SustainBudget::default();
        let start = Instant::now();
        cap(&mut budget, start);
        cap(&mut budget, start + Duration::from_secs(10));

        let later = start + Duration::from_secs(10) + REENTRY_WINDOW + Duration::from_secs(1);
        assert_eq!(None, budget.episode_duration(later));
        assert_eq!(1_600, cap(&mut budget, later));
    }
}
