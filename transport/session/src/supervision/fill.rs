//! The PIX fill planner — the pure rate law behind the Exit's own keep-alives.
//!
//! A PIX Exit is paid per return packet: each Exit → Entry packet spends one SURB carrying one
//! share, and a funded cycle only recovers once its whole emission has gone back. Nothing in the
//! protocol makes the *application* send those packets, so an idle Session strands the deposit it has
//! already been paid. The Exit therefore sends its own, at a rate this module derives once per
//! [`SAMPLING_INTERVAL`] from three quantities and nothing else:
//!
//! ```text
//! remaining  = (E − shares seen on the target cycle) × (1 + loss_margin)
//! finish_by  = target.hard_deadline − (1 − finish_fraction) × max_recovery_time
//! required   = remaining / max(finish_by − now, SAMPLING_INTERVAL)          packets/s
//! organic    = EMA of the gate's served counter over ORGANIC_WINDOW         packets/s
//! fill       = clamp(required − organic, heartbeat, max_rate)
//! ```
//!
//! Everything here is pure: no clock is read, no I/O is performed, and every input arrives as an
//! argument. The supervisor owns the target selection — which cycle the Exit is currently working
//! for — because that is a lifecycle question; this module owns only the arithmetic and the
//! decision of whether the answer is worth emitting.
//!
//! # Why `organic` is the gate counter
//!
//! `served_total` counts packets that passed the [`ServiceGate`](super::gate::ServiceGate), which is
//! every application data packet and nothing else — fill itself is ungated, exactly as the SURB-level
//! keep-alives it reuses are. So the counter measures what the application is contributing and never
//! what this planner is, which is what makes `required − organic` a subtraction rather than a
//! feedback loop that would drive itself to zero.
//!
//! # Why the stall rule cannot be the idle rule
//!
//! [`SupervisorConfig::max_recovery_idle`] closes a Session whose cycle stops progressing *while
//! service is being consumed*, and it reads the same gate counter. Fill bypasses the gate, so a cycle
//! that is being filled and making no progress is invisible to it: the idle rule keeps re-arming
//! because, as far as it can see, nothing was served. Left alone, an Exit would fill a doomed cycle —
//! one whose polynomial failed, or whose Entry is supplying SURBs that carry no shares — at
//! `max_rate` for the whole of `max_recovery_time`. The rule below is the bound that the gate-based
//! one cannot supply: once the target has not moved for `max_recovery_idle`, fill falls back to the
//! heartbeat and stays there until the cycle moves again.

use std::time::{Duration, Instant};

use hopr_api::types::internal::prelude::HoprPseudonym;
use hopr_protocol_pix::{PixParams, SsaId};

use super::{FillRate, ORGANIC_WINDOW, PixFillConfig, SAMPLING_INTERVAL, SupervisorConfig};

/// The cycle the Exit is currently filling for, as the supervisor sees it.
///
/// Deliberately a snapshot rather than a borrow of the supervisor's own record: the planner must not
/// be able to reach a cycle's lifecycle state, only the three numbers the rate law is a function of.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FillTarget {
    /// Which cycle this is, so the planner can tell a stall from a handoff.
    pub ssa_id: SsaId<HoprPseudonym>,
    /// Shares the cycle has accepted so far — surplus included, since every emitted share rides one
    /// packet and it is packets that are being planned.
    pub largest_shares_seen: u64,
    /// The immutable per-cycle recovery deadline this cycle must finish before.
    pub hard_deadline: Instant,
    /// Whether this target is a post-close drain, so the whole remainder is wanted at once.
    ///
    /// A live Session is filled at the pace its deadline needs, because it will still be there when
    /// the deadline arrives. A drained one will not be: it is closed, the buffer it is spending is
    /// finite and nothing is refilling it, so the only sensible rate is as fast as
    /// [`PixFillConfig::max_rate`] allows. See "Draining a closed Session" in the module
    /// documentation of `supervision`.
    pub drain: bool,
}

/// The pure rate law. See the module documentation for what it computes and why.
pub(crate) struct FillPlanner {
    cfg: PixFillConfig,
    /// [`SupervisorConfig::max_recovery_time`], the budget `finish_fraction` is a fraction of.
    max_recovery_time: Duration,
    /// [`SupervisorConfig::max_recovery_idle`], reused as the stall horizon so that fill gives up on
    /// a motionless cycle at the same point the idle rule would have, had it been able to see it.
    max_recovery_idle: Duration,
    /// `E` — every share one cycle of the negotiated dimensions emits, and therefore every packet it
    /// takes to complete one.
    cycle_shares: u64,
    /// When [`plan`](Self::plan) last ran. Also the base of [`next_tick`](Self::next_tick).
    last_tick: Instant,
    /// The gate's served counter at `last_tick`, so the next tick can difference it.
    last_served_total: u64,
    /// Exponential average of organic egress, in packets per second.
    organic_pps: f64,
    /// The cycle the counters below belong to, so a handoff resets them rather than reading the
    /// successor's progress as the predecessor's stall.
    target: Option<SsaId<HoprPseudonym>>,
    /// `largest_shares_seen` as of the last tick that observed movement.
    target_seen: u64,
    /// When the target last moved, or when it became the target. `None` while there is no target.
    target_progress_at: Option<Instant>,
    /// The rate the stream is believed to be running at, and the value hysteresis compares against.
    last_rate: FillRate,
    /// Whether the stall has already been reported, so a stalled cycle warns once rather than once a
    /// second for the whole of `max_recovery_time`.
    stall_warned: bool,
    /// Set on the tick a stall begins, and taken by the worker so it can count the event.
    ///
    /// An edge rather than a level, because the metric it feeds counts stalls: a level would count
    /// one per second for as long as the cycle stayed motionless, which is a measure of how long the
    /// operator waited rather than of how often this happened.
    stall_onset: bool,
}

impl FillPlanner {
    /// Builds a planner for a Session of the given dimensions.
    ///
    /// `now` seeds [`next_tick`](Self::next_tick), so the first plan happens one sampling interval
    /// after the Session starts rather than immediately. Nothing is lost by that: a Session has no
    /// funded cycle to fill for until at least a commitment and a deposit have crossed the wire.
    pub(crate) fn new(cfg: &SupervisorConfig, dims: &PixParams, now: Instant) -> Self {
        Self {
            cfg: cfg.fill.clone(),
            max_recovery_time: cfg.max_recovery_time,
            max_recovery_idle: cfg.max_recovery_idle,
            cycle_shares: dims.polys_per_ssa() as u64 * dims.emitted_shares_per_poly() as u64,
            last_tick: now,
            last_served_total: 0,
            organic_pps: 0.0,
            target: None,
            target_seen: 0,
            target_progress_at: None,
            last_rate: FillRate::ZERO,
            stall_warned: false,
            stall_onset: false,
        }
    }

    /// Whether fill is configured on at all.
    pub(crate) fn is_enabled(&self) -> bool {
        self.cfg.enabled
    }

    /// Reports, once, that the target cycle has gone motionless and fill has fallen back.
    ///
    /// Consumed rather than read so the caller counts the stall rather than its duration. The caller
    /// is the worker, which is the layer allowed to have side effects; this module stays pure.
    pub(crate) fn take_stall_onset(&mut self) -> bool {
        std::mem::take(&mut self.stall_onset)
    }

    /// Whether the stream this planner drives is currently emitting nothing.
    ///
    /// The supervisor uses it to decide whether a tick is worth waking for when there is no target:
    /// a planner that has already stopped has nothing left to say, while one that is still running
    /// owes the stream a zero.
    pub(crate) fn is_idle(&self) -> bool {
        self.last_rate.is_zero()
    }

    /// When the next plan is due.
    pub(crate) fn next_tick(&self) -> Instant {
        self.last_tick.checked_add(SAMPLING_INTERVAL).unwrap_or(self.last_tick)
    }

    /// Silences the stream, if it is running, and forgets the target.
    ///
    /// Returns the action's payload only when there is something to stop, so a Session that never
    /// filled — a disabled one, or one that closed before its first cycle funded — emits nothing and
    /// its action stream is byte-for-byte what it was before fill existed.
    pub(crate) fn stop(&mut self) -> Option<FillRate> {
        self.target = None;
        self.target_seen = 0;
        self.target_progress_at = None;
        self.stall_warned = false;
        (!self.last_rate.is_zero()).then(|| {
            self.last_rate = FillRate::ZERO;
            FillRate::ZERO
        })
    }

    /// Re-derives the fill rate, returning it only if it is worth emitting.
    ///
    /// `served_total` is the service gate's counter — organic egress and nothing else. `target` is
    /// the cycle the Exit is working for, or `None` when it has none, which is the case that stops
    /// the stream.
    pub(crate) fn plan(&mut self, now: Instant, served_total: u64, target: Option<FillTarget>) -> Option<FillRate> {
        if !self.cfg.enabled {
            return None;
        }

        self.observe_organic(now, served_total);

        let Some(target) = target else {
            return self.stop();
        };

        self.observe_target(now, &target);

        let required = self.required_rate(now, &target);
        let wanted = (required - self.organic_pps).max(0.0);
        let rate = self.bound(now, wanted);

        self.should_emit(rate).then(|| {
            tracing::debug!(
                ssa_id = %target.ssa_id,
                seen = target.largest_shares_seen,
                cycle_shares = self.cycle_shares,
                required,
                organic = self.organic_pps,
                packets = rate.packets,
                per = ?rate.per,
                "planned a new PIX fill rate"
            );
            self.last_rate = rate;
            rate
        })
    }

    /// Folds this interval's organic egress into the exponential average.
    ///
    /// `alpha = dt / ORGANIC_WINDOW`, capped at one, so the average has the same time constant
    /// however irregularly the ticks land — a worker that was busy for five seconds contributes five
    /// seconds of weight, not one tick's worth. A tick with no elapsed time contributes nothing
    /// rather than dividing by zero.
    fn observe_organic(&mut self, now: Instant, served_total: u64) {
        let dt = now.saturating_duration_since(self.last_tick).as_secs_f64();
        if dt > 0.0 {
            // Saturating because the gate counter is monotonic, so a decrease can only mean the
            // sample was taken across a reset; reading it as a huge positive delta would suppress
            // fill for the whole of the averaging window.
            let sample = served_total.saturating_sub(self.last_served_total) as f64 / dt;
            let alpha = (dt / ORGANIC_WINDOW.as_secs_f64()).min(1.0);
            self.organic_pps += alpha * (sample - self.organic_pps);
        }
        self.last_served_total = served_total;
        self.last_tick = now;
    }

    /// Tracks whether the target has moved, which is the input to the stall rule.
    ///
    /// A change of target resets the clock rather than inheriting the predecessor's: a successor that
    /// has just reached the front has made no progress *yet*, and reading that as a stall would hold
    /// every cycle after the first at the heartbeat.
    fn observe_target(&mut self, now: Instant, target: &FillTarget) {
        if self.target != Some(target.ssa_id) {
            self.target = Some(target.ssa_id);
            self.target_seen = target.largest_shares_seen;
            self.target_progress_at = Some(now);
            self.stall_warned = false;
        } else if target.largest_shares_seen > self.target_seen {
            self.target_seen = target.largest_shares_seen;
            self.target_progress_at = Some(now);
            self.stall_warned = false;
        }
    }

    /// The rate at which the cycle's remaining emission clears the aim point, in packets per second.
    ///
    /// The aim point sits `(1 − finish_fraction) × max_recovery_time` before the hard deadline, so
    /// the last part of the budget is margin: for loss beyond `loss_margin`, for a mixnet delay
    /// spike, and for the commitment and deposit round trip the successor still needs. An aim point
    /// already in the past collapses to one sampling interval, which asks for the whole remainder at
    /// once and is then bounded by `max_rate` — the correct behaviour for a cycle that is out of time,
    /// since there is no rate at which it can still be saved and no reason to stop trying.
    ///
    /// A [drain](FillTarget::drain) collapses the horizon the same way for a different reason: pacing
    /// a cycle to its deadline only makes sense while the Session is still there to be paced, and a
    /// drained one is spending a buffer nothing will refill. The ceiling in
    /// [`bound`](Self::bound) then becomes the only thing setting the rate, which is the intent
    /// rather than a side effect.
    fn required_rate(&self, now: Instant, target: &FillTarget) -> f64 {
        let remaining =
            self.cycle_shares.saturating_sub(target.largest_shares_seen) as f64 * (1.0 + self.cfg.loss_margin);
        let horizon = if target.drain {
            SAMPLING_INTERVAL
        } else {
            let slack = self.max_recovery_time.mul_f64(1.0 - self.cfg.finish_fraction);
            target
                .hard_deadline
                .checked_sub(slack)
                .map(|finish_by| finish_by.saturating_duration_since(now))
                .unwrap_or(SAMPLING_INTERVAL)
                .max(SAMPLING_INTERVAL)
        };
        remaining / horizon.as_secs_f64()
    }

    /// Applies the stall rule and the two bounds, and expresses the result as a [`FillRate`].
    ///
    /// The heartbeat is a floor rather than a special case, so a Session whose application is keeping
    /// up still emits one packet per period: it refreshes the Entry's idle eviction, it carries the
    /// SURB level the Entry's balancer acts on, and it is what distinguishes a planner that decided
    /// on nothing from one that has stopped planning.
    fn bound(&mut self, now: Instant, wanted: f64) -> FillRate {
        let heartbeat = FillRate::once_per(self.cfg.heartbeat);
        let heartbeat_pps = heartbeat.as_packets_per_sec();
        let ceiling = self.cfg.max_rate as f64;

        let stalled = self
            .target_progress_at
            .is_some_and(|at| now.saturating_duration_since(at) >= self.max_recovery_idle);
        let wanted = if stalled && wanted > heartbeat_pps {
            if !self.stall_warned {
                self.stall_warned = true;
                self.stall_onset = true;
                tracing::warn!(
                    ssa_id = ?self.target,
                    seen = self.target_seen,
                    cycle_shares = self.cycle_shares,
                    stalled_for = ?self.max_recovery_idle,
                    "PIX fill fell back to its heartbeat: the cycle it is filling for has stopped progressing"
                );
            }
            heartbeat_pps
        } else {
            wanted
        };

        // `clamp` would panic on an inverted pair, and the pair is invertible: a heartbeat period
        // shorter than `1 / max_rate` puts the floor above the ceiling. That configuration is
        // nonsense rather than impossible, and the ceiling is the one that has to win — it is the
        // bound on what this node will put on the wire.
        let bounded = wanted.max(heartbeat_pps.min(ceiling)).min(ceiling);

        if bounded >= 1.0 {
            // Rounding up rather than down: a rate rounded down never finishes the cycle, which is
            // the one outcome the whole mechanism exists to prevent. The cast is bounded by the
            // clamp above, and the explicit floor keeps the cast honest.
            FillRate::per_second((bounded.ceil() as u32).clamp(1, self.cfg.max_rate))
        } else if bounded <= heartbeat_pps {
            heartbeat
        } else {
            // Between the heartbeat and one packet a second, the rate is expressed as a *period*
            // instead of being rounded up to 1/s. Rounding here is not a small error: a cycle with a
            // few hundred packets left over an hour needs about a packet a minute, and quoting that
            // as one a second is sixty times the traffic — spent, moreover, on the tail of a cycle
            // that has nearly finished. `FillRate` carries its own unit precisely so this case is
            // exact. The `min` is a guard rather than a rule: `bounded` is at least `heartbeat_pps`
            // here, so the period is already at most the heartbeat's.
            FillRate::once_per(Duration::from_secs_f64(
                (1.0 / bounded).min(self.cfg.heartbeat.as_secs_f64()),
            ))
        }
    }

    /// Whether `rate` differs from what the stream is already doing by enough to be worth an action.
    ///
    /// The planner runs every second for the whole life of a cycle — hours, at production dimensions
    /// — and the required rate drifts continuously as the remainder and the horizon both shrink.
    /// Emitting every one of those would put a per-second action on a channel sized for lifecycle
    /// transitions, and the driver would reconfigure a rate controller for a change no packet
    /// schedule can express. So a change is reported when it is a real one:
    ///
    /// * any change to or from zero, because that is the difference between filling and not;
    /// * any crossing of the heartbeat, because that is the difference between filling and idling;
    /// * otherwise, a relative change of more than ten per cent.
    fn should_emit(&self, rate: FillRate) -> bool {
        if rate == self.last_rate {
            return false;
        }
        if rate.is_zero() || self.last_rate.is_zero() {
            return true;
        }

        let heartbeat = FillRate::once_per(self.cfg.heartbeat);
        if (rate == heartbeat) != (self.last_rate == heartbeat) {
            return true;
        }

        let old = self.last_rate.as_packets_per_sec();
        let new = rate.as_packets_per_sec();
        // `old` is non-zero here, both arms above having returned on a zero rate.
        (new - old).abs() > 0.1 * old
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use hopr_api::types::crypto_random::Randomizable;
    use hopr_protocol_pix::SsaIndex;

    use super::*;

    /// The three fields [`FillPlanner::bound`] reads, and nothing else that matters to it.
    fn cfg(heartbeat: Duration, max_rate: u32, max_recovery_idle: Duration) -> SupervisorConfig {
        SupervisorConfig {
            max_recovery_idle,
            // Only has to stay above the idle deadline; `bound` never reads it.
            max_recovery_time: max_recovery_idle * 2,
            fill: PixFillConfig {
                heartbeat,
                max_rate,
                ..PixFillConfig::default()
            },
            ..SupervisorConfig::default()
        }
    }

    /// See the identically-named helper in `super::supervisor` for why the surplus is non-zero.
    /// `bound` reads none of it, but `FillPlanner::new` needs dimensions to size a cycle.
    fn dims() -> PixParams {
        PixParams::try_new(10, 5, 7, crate::types::LOCAL_PIX_SUITE).expect("test dimensions must be valid")
    }

    fn target(now: Instant) -> FillTarget {
        FillTarget {
            ssa_id: SsaId::new(
                HoprPseudonym::random(),
                SsaIndex::new(1).expect("index one is non-zero"),
            ),
            largest_shares_seen: 0,
            hard_deadline: now + Duration::from_secs(3600),
            drain: false,
        }
    }

    /// An inverted heartbeat/ceiling pair resolves ceiling-wins rather than panicking.
    ///
    /// A heartbeat period shorter than `1 / max_rate` puts the floor above the ceiling, which is
    /// nonsense rather than impossible — nothing rejects the pair, since each field is legal on its
    /// own. `f64::clamp` panics on it. The ceiling has to be the one that survives: it is the bound
    /// on what this node will put on the wire, and a floor that overrode it would turn the one guard
    /// on self-generated egress into its opposite.
    #[test]
    fn an_inverted_heartbeat_and_ceiling_resolve_to_the_ceiling() {
        let now = Instant::now();
        let mut planner = FillPlanner::new(
            &cfg(Duration::from_millis(10), 5, Duration::from_secs(30)),
            &dims(),
            now,
        );

        // The heartbeat asks for 100 packets/s against a ceiling of five, from both directions: with
        // nothing wanted, so only the floor is in play, and with more wanted than the ceiling allows.
        for wanted in [0.0, 50.0] {
            let rate = planner.bound(now, wanted);
            assert_eq!(
                FillRate::per_second(5),
                rate,
                "an inverted pair must resolve to the ceiling, got {rate:?} for a wanted rate of {wanted}"
            );
        }
    }

    /// Fractional rates round *up*, and the ceiling still binds after the rounding.
    ///
    /// Rounding down is the one direction that cannot be tolerated: a rate a hair below what the
    /// cycle needs never finishes it, which is the whole outcome this mechanism exists to prevent.
    /// The ceiling is applied before the rounding, so a `max_rate` of ten can never be rounded to
    /// eleven.
    #[test]
    fn fractional_rates_round_up_and_stay_under_the_ceiling() {
        let now = Instant::now();
        let mut planner = FillPlanner::new(&cfg(Duration::from_secs(60), 10, Duration::from_secs(30)), &dims(), now);

        assert_eq!(FillRate::per_second(5), planner.bound(now, 4.2));
        assert_eq!(FillRate::per_second(10), planner.bound(now, 9.001));
        assert_eq!(
            FillRate::per_second(10),
            planner.bound(now, 10.4),
            "the ceiling must bind before the rounding, not after it"
        );
        assert_eq!(
            FillRate::per_second(10),
            planner.bound(now, f64::MAX),
            "no wanted rate may produce more than the ceiling"
        );
    }

    /// Between the heartbeat and one packet a second, the rate is a *period* rather than a rounding.
    ///
    /// A cycle with a few hundred packets left over an hour needs about a packet a minute. Rounding
    /// that up to one a second is sixty times the traffic, spent on the tail of a cycle that has
    /// nearly finished — so [`FillRate`] carries its own unit and this range uses it. The period must
    /// never come out longer than the heartbeat, which is the floor.
    #[test]
    fn sub_hertz_rates_are_expressed_as_a_period_no_longer_than_the_heartbeat() {
        let heartbeat = Duration::from_secs(60);
        let now = Instant::now();
        let mut planner = FillPlanner::new(&cfg(heartbeat, 250, Duration::from_secs(30)), &dims(), now);

        let rate = planner.bound(now, 0.1);
        assert_eq!(1, rate.packets, "a sub-hertz rate must stay one packet per period");
        assert_eq!(Duration::from_secs(10), rate.per);

        // Just above the heartbeat, where the `min` in that branch is closest to binding.
        let barely = planner.bound(now, 1.0 / 50.0);
        assert_eq!(1, barely.packets);
        assert!(
            barely.per <= heartbeat,
            "a period longer than the heartbeat would put the rate under its own floor, got {:?}",
            barely.per
        );

        // And nothing wanted at all is the heartbeat itself rather than silence.
        assert_eq!(FillRate::once_per(heartbeat), planner.bound(now, 0.0));
    }

    /// A motionless target drops fill to the heartbeat, and to no more than the ceiling.
    ///
    /// This is the bound the gate-based idle rule cannot supply: fill bypasses the service gate, so a
    /// cycle being filled and making no progress looks perfectly quiet to `max_recovery_idle` and it
    /// re-arms forever. Without this rule a doomed cycle — a failed polynomial, or an Entry supplying
    /// SURBs that carry no shares — would be filled at `max_rate` for the whole of
    /// `max_recovery_time`.
    #[test]
    fn a_stalled_target_falls_back_to_the_heartbeat() {
        let heartbeat = Duration::from_secs(60);
        let idle = Duration::from_secs(30);
        let now = Instant::now();
        let mut planner = FillPlanner::new(&cfg(heartbeat, 250, idle), &dims(), now);

        planner.observe_target(now, &target(now));
        assert_eq!(
            FillRate::per_second(200),
            planner.bound(now + idle - Duration::from_secs(1), 200.0),
            "a target that has moved inside the idle window is not stalled"
        );
        assert!(!planner.take_stall_onset(), "no stall has begun yet");

        let stalled = planner.bound(now + idle, 200.0);
        assert_eq!(FillRate::once_per(heartbeat), stalled);
        assert!(
            stalled.as_packets_per_sec() <= FillRate::once_per(heartbeat).as_packets_per_sec(),
            "the stall fallback must never exceed the heartbeat rate"
        );
        assert!(planner.take_stall_onset(), "the stall must be reported exactly once");
        assert!(!planner.take_stall_onset(), "and not once per tick thereafter");

        // With an inverted pair the ceiling is below the heartbeat, and it still wins on this path.
        let now = Instant::now();
        let mut inverted = FillPlanner::new(&cfg(Duration::from_millis(10), 5, idle), &dims(), now);
        inverted.observe_target(now, &target(now));
        let stalled = inverted.bound(now + idle, 200.0);
        assert_eq!(FillRate::per_second(5), stalled);
        assert!(
            stalled.as_packets_per_sec() <= 5.0,
            "the stall fallback must never exceed the ceiling either"
        );
    }

    /// A post-close drain wants its whole remainder at once, and `max_rate` is what paces it.
    ///
    /// The ordinary rate law spreads the remainder over the aim point, which is the right answer for
    /// a Session that will still be there in an hour. A drained Session will not be: it is closed,
    /// the buffer it is spending is finite and nothing is refilling it, and every SURB in it that is
    /// not spent on a share is one the deposit being recovered has already paid for. So the horizon
    /// collapses to a single sampling interval and the ceiling becomes the only thing setting the
    /// pace — which is the intended reading of "drain", not an accident of the arithmetic.
    #[test]
    fn a_draining_target_asks_for_its_whole_remainder_at_once() {
        let idle = Duration::from_secs(30);
        let now = Instant::now();
        let mut planner = FillPlanner::new(&cfg(Duration::from_secs(60), 250, idle), &dims(), now);

        // An hour of runway on both, so the only difference between them is the flag.
        let patient = target(now);
        let draining = FillTarget { drain: true, ..patient };

        let whole_remainder = planner.cycle_shares as f64 * (1.0 + planner.cfg.loss_margin);
        let drained = planner.required_rate(now, &draining);
        assert!(
            (drained - whole_remainder / SAMPLING_INTERVAL.as_secs_f64()).abs() < 1e-9,
            "a drain must ask for its whole remainder inside one sampling interval, got {drained} against a remainder \
             of {whole_remainder}"
        );

        let patient_rate = planner.required_rate(now, &patient);
        assert!(
            patient_rate < 1.0 && patient_rate * 1_000.0 < drained,
            "the same target with an hour of runway must still be spread over that hour, got {patient_rate}"
        );

        // Below the ceiling the drain is the remainder itself, rounded up as every rate is.
        assert_eq!(
            FillRate::per_second(whole_remainder.ceil() as u32),
            planner.bound(now, drained)
        );

        // And above it, the ceiling binds — the drain is loud, not unbounded.
        let mut capped = FillPlanner::new(&cfg(Duration::from_secs(60), 100, idle), &dims(), now);
        assert_eq!(FillRate::per_second(100), capped.bound(now, drained));
    }
}
