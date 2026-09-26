//! Backing keep-alive production off while the local uplink is queueing.
//!
//! Keep-alives are the one part of the Entry's upstream that exists only for the balancer, so they
//! are what gives way when the uplink cannot keep up. Nothing else in the balancer can see that:
//! `max_surbs_per_sec` is a configured budget, not a measurement, and a budget sized for a fast
//! link overloads a slow one. At 5063 SURB/s the keep-alives alone are 29.6 Mbit/s. On a
//! 10 Mbit/s uplink the local queues then grow by about 1700 packets per second, and every probe
//! and reply waits behind them.
//!
//! The ceiling is lowered multiplicatively while the transport reports a standing egress queue
//! (see [`EgressPressure`](crate::egress::EgressPressure)), and raised additively once the queue
//! has cleared, as TCP does with its window. It never exceeds the configured limit, and without
//! congestion it simply is that limit.

use std::time::{Duration, Instant};

/// Factor the ceiling is multiplied by on each backoff.
const BACKOFF_FACTOR: f64 = 0.7;

/// Shortest time between two backoffs.
///
/// A lower rate takes a while to drain the queue it was backing off from, and the transport keeps
/// reporting that queue until it has drained. Backing off on every report would push the ceiling
/// far below what the uplink carries.
const BACKOFF_SPACING: Duration = Duration::from_millis(500);

/// How long egress must stay clear of congestion before the ceiling starts rising again.
const QUIET_BEFORE_PROBING: Duration = Duration::from_secs(1);

/// How fast the ceiling rises once egress is clear, as a fraction of the configured limit per
/// second. From the floor back to the limit this takes about twenty seconds.
const PROBE_FRACTION_PER_SEC: f64 = 0.05;

/// Lowest ceiling backoff can reach, in SURBs per second.
///
/// 50 keep-alive packets per second, about 0.6 Mbit/s: enough to keep a Session alive and to
/// notice when the uplink recovers, never enough to congest it.
const FLOOR_SURBS_PER_SEC: f64 = 100.0;

/// The keep-alive ceiling as backed off from local egress congestion.
#[derive(Clone, Debug, Default)]
pub(crate) struct CongestionCeiling {
    /// The backed-off ceiling, or `None` while no congestion has held it below the limit.
    ceiling: Option<f64>,
    /// When the ceiling was last lowered.
    last_backoff: Option<Instant>,
}

impl CongestionCeiling {
    /// Advances the ceiling by one sample of length `dt` ending at `now`, and returns it.
    ///
    /// `last_congested` is when the transport last reported a standing egress queue. The result
    /// never exceeds `limit`.
    pub(crate) fn update(&mut self, now: Instant, dt: Duration, last_congested: Option<Instant>, limit: u64) -> u64 {
        let limit = limit as f64;
        let floor = FLOOR_SURBS_PER_SEC.min(limit);

        let reported_since_backoff = last_congested.is_some_and(|at| self.last_backoff.is_none_or(|b| at > b));
        let may_back_off = self
            .last_backoff
            .is_none_or(|b| now.saturating_duration_since(b) >= BACKOFF_SPACING);
        let quiet = last_congested.is_none_or(|at| now.saturating_duration_since(at) >= QUIET_BEFORE_PROBING);

        if reported_since_backoff && may_back_off {
            let current = self.ceiling.unwrap_or(limit).min(limit);
            self.ceiling = Some((current * BACKOFF_FACTOR).max(floor));
            self.last_backoff = Some(now);
        } else if quiet && let Some(ceiling) = self.ceiling {
            let raised = ceiling + limit * PROBE_FRACTION_PER_SEC * dt.as_secs_f64();
            // Back at the limit, stop tracking: a limit raised later then applies at once.
            self.ceiling = (raised < limit).then_some(raised);
        }

        self.ceiling.map_or(limit, |c| c.min(limit)).round() as u64
    }

    /// Whether congestion is currently holding the ceiling below the limit.
    pub(crate) fn is_backed_off(&self) -> bool {
        self.ceiling.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIMIT: u64 = 5_000;
    const TICK: Duration = Duration::from_millis(100);

    /// Steps a ceiling on a synthetic clock, with the transport reporting congestion on the ticks
    /// the caller says.
    struct Clock {
        ceiling: CongestionCeiling,
        now: Instant,
        last_congested: Option<Instant>,
    }

    impl Clock {
        fn new() -> Self {
            Self {
                ceiling: CongestionCeiling::default(),
                now: Instant::now(),
                last_congested: None,
            }
        }

        fn tick(&mut self, congested: bool) -> u64 {
            self.now += TICK;
            if congested {
                self.last_congested = Some(self.now);
            }
            self.ceiling.update(self.now, TICK, self.last_congested, LIMIT)
        }

        fn run(&mut self, ticks: usize, congested: bool) -> u64 {
            (0..ticks).fold(0, |_, _| self.tick(congested))
        }
    }

    #[test]
    fn without_congestion_the_ceiling_should_be_the_limit() {
        let mut clock = Clock::new();
        assert_eq!(LIMIT, clock.run(50, false));
        assert!(!clock.ceiling.is_backed_off());
    }

    #[test]
    fn congestion_should_back_the_ceiling_off_multiplicatively() {
        let mut clock = Clock::new();
        assert_eq!(3_500, clock.tick(true));
        assert!(clock.ceiling.is_backed_off());
    }

    /// The transport reports every packet that waited while the queue drains, so a burst of
    /// reports is one congestion event, not one per report.
    #[test]
    fn backoff_should_happen_at_most_once_per_spacing() {
        let mut clock = Clock::new();
        let ticks_per_spacing = (BACKOFF_SPACING.as_millis() / TICK.as_millis()) as usize;

        let first = clock.tick(true);
        let within_spacing = clock.run(ticks_per_spacing - 1, true);
        assert_eq!(
            first, within_spacing,
            "reports within the spacing must not back off again"
        );

        let after_spacing = clock.tick(true);
        assert!(
            after_spacing < within_spacing,
            "a report after the spacing must back off again"
        );
    }

    #[test]
    fn sustained_congestion_should_stop_at_the_floor() {
        let mut clock = Clock::new();
        assert_eq!(FLOOR_SURBS_PER_SEC as u64, clock.run(200, true));
    }

    #[test]
    fn the_floor_should_never_exceed_the_limit() {
        let mut ceiling = CongestionCeiling::default();
        let now = Instant::now();
        assert_eq!(40, ceiling.update(now, TICK, Some(now), 40));
    }

    /// Probing up right after a backoff would re-fill the queue that is still draining.
    #[test]
    fn the_ceiling_should_hold_until_egress_has_been_quiet() {
        let mut clock = Clock::new();
        let backed_off = clock.tick(true);
        let quiet_ticks = (QUIET_BEFORE_PROBING.as_millis() / TICK.as_millis()) as usize;

        assert_eq!(
            backed_off,
            clock.run(quiet_ticks - 1, false),
            "must hold while the queue drains"
        );
        assert!(
            clock.tick(false) > backed_off,
            "must probe up once egress has been quiet"
        );
    }

    #[test]
    fn a_clear_uplink_should_recover_the_limit_additively() {
        let mut clock = Clock::new();
        clock.run(40, true);

        let mut previous = clock.tick(false);
        let step_bound = (LIMIT as f64 * PROBE_FRACTION_PER_SEC * TICK.as_secs_f64()).ceil() as u64 + 1;
        for _ in 0..400 {
            let next = clock.tick(false);
            assert!(next >= previous, "recovery must not back off without congestion");
            assert!(
                next - previous <= step_bound,
                "recovery must be additive: {previous} -> {next}"
            );
            previous = next;
        }
        assert_eq!(LIMIT, previous, "a clear uplink must get the whole limit back");
        assert!(!clock.ceiling.is_backed_off());
    }

    /// The configured limit stays the hard ceiling: lowering it applies at once, whatever the
    /// backoff state.
    #[test]
    fn the_ceiling_should_never_exceed_a_lowered_limit() {
        let mut clock = Clock::new();
        clock.tick(true);
        let now = clock.now + TICK;
        assert_eq!(1_000, clock.ceiling.update(now, TICK, clock.last_congested, 1_000));
    }
}
