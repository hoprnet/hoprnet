//! Paced SURB refill: production follows consumption and closes the gap to target over a horizon.
//!
//! The controller this replaces on the Entry applied PID gains per sample, with no notion of
//! elapsed time. At the 100 ms default sampling interval, any deficit that lasted about a second
//! saturated its output at the ceiling. Above target, the integral wound down to minus the ceiling
//! and held production at zero for seconds. That made refills bursts at the full keep-alive budget:
//! thousands of packets per second, enough to saturate a laptop uplink and queue everything else
//! behind them.
//!
//! Here, production is the sum of two terms:
//!
//! - the consumption that organic supply does not cover, fed forward, so that holding the level costs no error at all;
//! - the gap to target spread over [`PacedRefillParams::refill_horizon`], and never more than the consumption itself
//!   (or a small idle floor), so that a refill asks for roughly as much again as the session already spends rather than
//!   the whole budget.
//!
//! The output then rises no faster than [`PacedRefillParams::ramp_time`] allows and never exceeds
//! the ceiling. It falls immediately.
//!
//! ## Pre-loading
//!
//! The one exception is a Session's first fill. Nothing can use the Session until the counterparty
//! holds half the target (the readiness wait in the session manager), and it holds nothing yet. So
//! until the level first reaches half the target, the gap is closed as fast as the ceiling allows,
//! with no ramp and no consumption bound -- the behaviour the previous controller had throughout.
//! This happens once per Session: later deficits, including the restart after a degraded return
//! path, are refilled paced.
//!
//! ## Convergence
//!
//! In estimate space the level moves by `organic + keep-alive − consumed`. With keep-alive
//! production of `ĉ + e/H`, where `ĉ` estimates `consumed − organic`, and `ĉ` settled at constant
//! consumption, that leaves `dL/dt = e/H`. The gap therefore decays as `e₀·e^(−t/H)`: no overshoot,
//! and no integral to wind up. A constant bias `b` in `ĉ` (for example the discount applied by SURB
//! decay) leaves a steady offset of `b·H`.
//!
//! After a drain to empty, production is paced and bounded by the ceiling `C`. The level reaches
//! 90 % of target `T` within about `T_ramp + 0.9·T / min(C − ĉ, R) + H·ln 10`, where `R` is the
//! refill allowance. If `C ≤ ĉ`, nothing refills it, whatever the controller. That is the uplink's
//! limit, and the Exit's own supply gating then slows the download.

use std::{str::FromStr, time::Duration};

use crate::balancer::{BalancerControllerBounds, ControlInput, SurbBalancerController};

/// Default time over which a gap to target is closed.
pub const DEFAULT_REFILL_HORIZON: Duration = Duration::from_secs(2);

/// Default shortest time in which production may rise from zero to the ceiling.
pub const DEFAULT_RAMP_TIME: Duration = Duration::from_secs(2);

/// How much a refill may add on top of the net consumption, as a multiple of it.
///
/// One means a refill never asks for more than doubling what the session already spends. A buffer
/// drained by a burst is then topped up about as fast as it is used, rather than at the full budget
/// while nothing is using it.
const REFILL_ALLOWANCE_PER_CONSUMPTION: f64 = 1.0;

/// Horizon of the refill allowance for a session that consumes nothing.
///
/// Scaling the allowance with consumption alone would never refill an idle session. This floor
/// refills an idle, drained buffer in about this long.
const IDLE_REFILL_HORIZON: Duration = Duration::from_secs(10);

/// Shortest refill horizon honoured. A zero horizon would ask for the whole gap at once.
const MIN_REFILL_HORIZON: Duration = Duration::from_millis(100);

/// Horizon over which the gap is closed while pre-loading: a couple of sampling intervals, so the
/// ceiling is what limits the first fill.
const PRELOAD_HORIZON: Duration = Duration::from_millis(250);

/// Timing parameters of a [`PacedRefillController`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PacedRefillParams {
    /// Time constant over which a gap to target is closed.
    pub refill_horizon: Duration,
    /// Shortest time in which production may rise from zero to the ceiling. Zero disables the
    /// limit.
    pub ramp_time: Duration,
}

impl Default for PacedRefillParams {
    fn default() -> Self {
        Self {
            refill_horizon: DEFAULT_REFILL_HORIZON,
            ramp_time: DEFAULT_RAMP_TIME,
        }
    }
}

impl PacedRefillParams {
    /// Uses `HOPR_BALANCER_REFILL_HORIZON_MS` and `HOPR_BALANCER_RAMP_MS` if set, otherwise the
    /// defaults.
    pub fn from_env_or_default() -> Self {
        let millis = |var: &str| {
            std::env::var(var)
                .ok()
                .and_then(|v| u64::from_str(v.trim()).ok())
                .map(Duration::from_millis)
        };
        let default = Self::default();
        Self {
            refill_horizon: millis("HOPR_BALANCER_REFILL_HORIZON_MS").unwrap_or(default.refill_horizon),
            ramp_time: millis("HOPR_BALANCER_RAMP_MS").unwrap_or(default.ramp_time),
        }
    }
}

/// [`SurbBalancerController`] that feeds consumption forward and paces refills.
///
/// See the [module documentation](self) for the control law and its bounds.
#[derive(Clone, Debug)]
pub struct PacedRefillController {
    bounds: BalancerControllerBounds,
    params: PacedRefillParams,
    /// Output of the previous sample, which the upward slew limit is measured from.
    last_output: f64,
    /// Whether the Session's first fill is still in progress. See the
    /// [module documentation](self#pre-loading).
    preloading: bool,
}

impl Default for PacedRefillController {
    fn default() -> Self {
        Self::new(PacedRefillParams::default())
    }
}

impl PacedRefillController {
    /// Creates an instance with the given timing parameters.
    ///
    /// Needs to be [reconfigured](SurbBalancerController::set_target_and_limit) in order to
    /// produce anything.
    pub fn new(params: PacedRefillParams) -> Self {
        Self {
            bounds: BalancerControllerBounds::default(),
            params,
            last_output: 0.0,
            preloading: true,
        }
    }

    /// How much above the net consumption a refill may ask for.
    fn refill_allowance(&self, net_consumption: f64) -> f64 {
        let idle_floor = self.bounds.target() as f64 / IDLE_REFILL_HORIZON.as_secs_f64();
        (REFILL_ALLOWANCE_PER_CONSUMPTION * net_consumption.max(0.0)).max(idle_floor)
    }
}

impl SurbBalancerController for PacedRefillController {
    fn bounds(&self) -> BalancerControllerBounds {
        self.bounds
    }

    /// Unlike a rebuild, this keeps the output the next sample slews from. A client ramping its
    /// target up step by step must not have production restart from zero on every step.
    fn set_target_and_limit(&mut self, bounds: BalancerControllerBounds) {
        self.bounds = bounds;
        self.last_output = self.last_output.min(bounds.output_limit() as f64);
    }

    fn next_control_output(&mut self, input: ControlInput) -> u64 {
        let ceiling = input.ceiling.min(self.bounds.output_limit()) as f64;
        let target = self.bounds.target();
        let gap = target as f64 - input.level as f64;

        // Mirrors the session manager's readiness wait: once half the target is held, the Session
        // is usable and the first fill is over.
        if self.preloading && target > 0 && input.level >= target / 2 {
            self.preloading = false;
        }

        if self.preloading {
            let wanted = input.net_consumption_per_sec + gap.max(0.0) / PRELOAD_HORIZON.as_secs_f64();
            let output = wanted.clamp(0.0, ceiling);
            self.last_output = output;
            return output.round() as u64;
        }

        let horizon = self.params.refill_horizon.max(MIN_REFILL_HORIZON).as_secs_f64();
        let correction = if gap > 0.0 {
            (gap / horizon).min(self.refill_allowance(input.net_consumption_per_sec))
        } else {
            gap / horizon
        };
        let wanted = input.net_consumption_per_sec + correction;

        let ramp = self.params.ramp_time.as_secs_f64();
        let rise_limit = if ramp > 0.0 {
            self.last_output + ceiling * input.dt.as_secs_f64() / ramp
        } else {
            f64::INFINITY
        };

        let output = wanted.min(rise_limit).clamp(0.0, ceiling);
        self.last_output = output;
        output.round() as u64
    }

    fn reset(&mut self) {
        // Nothing to discard: the output is computed afresh from the level and consumption handed
        // in. The previous output is kept on purpose, since it only limits how fast the output
        // rises, and dropping it would turn a change of regime into a restart from zero.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TICK: Duration = Duration::from_millis(50);

    fn controller(target: u64, limit: u64) -> PacedRefillController {
        let mut c = PacedRefillController::new(PacedRefillParams::default());
        c.set_target_and_limit(BalancerControllerBounds::new(target, limit));
        c
    }

    /// A controller past its Session's first fill, so that its next refill is paced.
    fn preloaded(target: u64, limit: u64) -> PacedRefillController {
        let mut c = controller(target, limit);
        c.next_control_output(input(target, 0.0));
        c
    }

    fn input(level: u64, net_consumption_per_sec: f64) -> ControlInput {
        ControlInput {
            level,
            net_consumption_per_sec,
            dt: TICK,
            ceiling: u64::MAX,
        }
    }

    /// Minimal closed loop: the level gains what was produced and loses what was consumed, both
    /// at the given per-second rates, one tick at a time.
    struct Loop {
        controller: PacedRefillController,
        level: f64,
        output: u64,
    }

    impl Loop {
        /// A Session opened with an empty buffer, pre-loading first.
        fn new(target: u64, limit: u64) -> Self {
            Self {
                controller: controller(target, limit),
                level: 0.0,
                output: 0,
            }
        }

        /// A Session past its first fill whose buffer has since drained to empty.
        fn drained(target: u64, limit: u64) -> Self {
            Self {
                controller: preloaded(target, limit),
                ..Self::new(target, limit)
            }
        }

        fn tick(&mut self, consumption: f64) -> u64 {
            let dt = TICK.as_secs_f64();
            self.level = (self.level + (self.output as f64 - consumption) * dt).max(0.0);
            self.output = self
                .controller
                .next_control_output(input(self.level.round() as u64, consumption));
            self.output
        }

        fn run(&mut self, ticks: u32, consumption: f64) -> u64 {
            (0..ticks).map(|_| self.tick(consumption)).max().unwrap_or(0)
        }
    }

    #[test]
    fn an_unconfigured_controller_should_produce_nothing() {
        let mut c = PacedRefillController::new(PacedRefillParams::default());
        assert_eq!(0, c.next_control_output(input(0, 100.0)));
    }

    /// The controller holds the level by replacing what is consumed, without needing an error to
    /// do it.
    #[test]
    fn at_target_the_output_should_equal_the_net_consumption() {
        let mut c = controller(1_000, 5_000);
        c.last_output = 800.0;
        assert_eq!(800, c.next_control_output(input(1_000, 800.0)));
    }

    #[test]
    fn the_output_should_rise_no_faster_than_the_ramp_allows() {
        let mut c = preloaded(10_000, 4_000);
        let per_tick = 4_000.0 * TICK.as_secs_f64() / DEFAULT_RAMP_TIME.as_secs_f64();

        let mut previous = 0;
        for _ in 0..20 {
            let output = c.next_control_output(input(0, 3_000.0));
            assert!(
                output as f64 <= previous as f64 + per_tick + 1.0,
                "rose from {previous} to {output}, more than {per_tick} per tick"
            );
            previous = output;
        }
    }

    #[test]
    fn the_output_should_fall_immediately() {
        let mut c = controller(1_000, 5_000);
        c.last_output = 3_000.0;
        assert_eq!(0, c.next_control_output(input(5_000, 0.0)));
    }

    #[test]
    fn the_output_should_never_exceed_the_limit_or_the_ceiling() {
        let mut c = controller(100_000, 2_000);
        let mut l = |ceiling| {
            (0..200)
                .map(|_| {
                    c.next_control_output(ControlInput {
                        ceiling,
                        ..input(0, 50_000.0)
                    })
                })
                .max()
                .unwrap_or(0)
        };
        assert_eq!(2_000, l(u64::MAX), "the limit caps the output");
        assert_eq!(700, l(700), "a lower ceiling caps it further, and at once");
    }

    /// A refill asks for about as much again as the session spends, not for the whole budget: the
    /// case that used to send thousands of keep-alives per second into a laptop uplink.
    #[test]
    fn a_refill_should_be_bounded_by_the_consumption_rather_than_the_ceiling() {
        let mut l = Loop::drained(9_766, 5_063);
        let peak = l.run(400, 1_000.0);
        assert!(
            peak <= 2_000,
            "refilling at 1000 SURB/s of consumption must not exceed twice that: peaked at {peak}"
        );
    }

    /// An idle session still refills, on the idle floor, rather than never.
    #[test]
    fn an_idle_drained_buffer_should_still_refill() {
        let mut l = Loop::drained(1_000, 5_000);
        l.run(2 * (IDLE_REFILL_HORIZON.as_millis() / TICK.as_millis()) as u32, 0.0);
        assert!(l.level >= 900.0, "idle refill stalled at {}", l.level);
    }

    /// The convergence claim: at constant consumption the level settles on target without
    /// overshooting it, within the analytic time bound -- from a Session's first fill as well as
    /// from a drain.
    #[test]
    fn the_level_should_converge_to_target_without_overshoot() {
        const TARGET: f64 = 1_000.0;
        for mut l in [Loop::new(TARGET as u64, 2_500), Loop::drained(TARGET as u64, 2_500)] {
            converge_without_overshoot(&mut l, TARGET, 800.0);
        }
    }

    fn converge_without_overshoot(l: &mut Loop, target: f64, consumption: f64) {
        let horizon = DEFAULT_REFILL_HORIZON.as_secs_f64();
        let bound = DEFAULT_RAMP_TIME.as_secs_f64() + horizon * 10f64.ln();
        let mut reached_90_at = None;
        let mut highest: f64 = 0.0;
        for n in 1..=400u32 {
            l.tick(consumption);
            highest = highest.max(l.level);
            if reached_90_at.is_none() && l.level >= 0.9 * target {
                reached_90_at = Some(n as f64 * TICK.as_secs_f64());
            }
        }

        let reached = reached_90_at.expect("the level must reach 90 % of target");
        assert!(
            reached <= bound,
            "reached 90 % after {reached:.2} s, bound {bound:.2} s"
        );
        assert!(highest <= target * 1.01, "overshot the target: {highest}");
        assert!((l.level - target).abs() <= target * 0.01, "settled at {}", l.level);
    }

    /// The previous controller wound its integral down while above target and then stayed silent
    /// for seconds once the level dropped below it. Without an integral, the first sample with a
    /// deficit already produces.
    #[test]
    fn a_long_stretch_above_target_should_not_delay_production_once_below_it() {
        let mut c = controller(1_000, 5_000);
        for _ in 0..600 {
            assert_eq!(0, c.next_control_output(input(5_000, 0.0)));
        }
        assert!(c.next_control_output(input(900, 100.0)) > 0);
    }

    /// A client ramping its target step by step reconfigures every second; each step must not
    /// throw away the output that production was slewing from.
    #[test]
    fn reconfiguration_should_keep_the_output_it_slews_from() {
        let mut c = preloaded(1_000, 5_000);
        for _ in 0..100 {
            c.next_control_output(input(0, 2_000.0));
        }
        let before = c.next_control_output(input(0, 2_000.0));

        c.set_target_and_limit(BalancerControllerBounds::new(1_300, 5_000));
        let after = c.next_control_output(input(0, 2_000.0));
        assert!(
            after >= before,
            "reconfiguring dropped the output from {before} to {after}"
        );

        c.set_target_and_limit(BalancerControllerBounds::new(1_300, 1_000));
        assert!(
            c.next_control_output(input(0, 2_000.0)) <= 1_000,
            "a lower limit applies at once"
        );
    }

    /// Nothing can use a Session until its first fill completes, so that fill runs at the ceiling
    /// from the first sample, as it did before pacing.
    #[test]
    fn pre_loading_should_fill_at_the_ceiling_without_a_ramp() {
        let mut c = controller(10_000, 4_000);
        assert_eq!(4_000, c.next_control_output(input(0, 0.0)));
    }

    /// The exception is the first fill only: once the Session is ready, a later drain -- including
    /// the restart after a degraded return path -- is refilled paced.
    #[test]
    fn pre_loading_should_end_once_the_session_is_ready_and_not_return() {
        let mut c = controller(10_000, 4_000);
        assert_eq!(
            4_000,
            c.next_control_output(input(4_999, 0.0)),
            "still below half the target"
        );

        c.next_control_output(input(5_000, 0.0));
        let refill = c.next_control_output(input(0, 500.0));
        // The consumption, plus an allowance of the larger of the consumption and target / 10 s.
        let paced_bound = 500 + 1_000;
        assert!(
            refill <= paced_bound,
            "a drain after readiness must be refilled paced, at most {paced_bound}/s rather than the 4000/s ceiling, \
             got {refill}"
        );
    }
}
