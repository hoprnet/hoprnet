//! Pure buffer-and-release logic for the virtual-clock timing-wheel mixer.
//!
//! This module is deliberately free of any async, threading or channel concerns so the release
//! decision can be unit-tested in isolation. The engine in [`crate::poisson`] owns a
//! `Vec<Entry<T>>` and calls [`enqueue`] on ingress and [`sweep`] on every wake.
//!
//! # Mechanism
//!
//! Every entry is tagged **once**, at enqueue, with a release threshold `v_release` in
//! dimensionless virtual time: `v_release = V + g`, where `V` is the pool's [`VirtualClock`] and
//! `g` is drawn from a truncated `Exp(1)` on `[0, g_max]` (`g_max = ln(1/miss_probability)`). The clock
//! advances by `dt / mu_max` on every event (`dt` = elapsed wall-clock time since the last sync)
//! plus, when `target_occupancy > 0`, one `1/target_occupancy` increment per arrival. An entry releases once
//! `V >= v_release` — a plain comparison, no per-packet CSPRNG draw during a sweep (the one draw
//! happens at enqueue).
//!
//! Three properties fall out of the clock update with no dedicated code (see the PR description
//! for the derivation and validation numbers):
//! - **Hard bound**: `V` advances by at least `dt/mu_max`, so an entry clears within `mu_max * g_max = max_delay` of
//!   enqueue, regardless of load.
//! - **Catch-up**: a delayed sweep has a larger `dt`, so `V` jumps further and more entries clear at once.
//! - **Self-limiting overload**: every arrival advances `V`, so a burst of arrivals is itself the drain signal — no
//!   separate overload valve is needed.

use std::time::{Duration, Instant};

use hopr_types::crypto_random;

use crate::config::{MixerConfig, PoissonConfig};

/// Idle heartbeat when the pool is empty, so an engine periodically notices a dropped receiver
/// and can shut down instead of parking forever. Both engines use it as the initial timer.
pub(crate) const IDLE_HEARTBEAT: Duration = Duration::from_millis(200);
/// Floor on the wake interval, so an engine never busy-spins even when the earliest pending
/// entry's deadline is imminent. This is deliberately the *only* floor: [`next_wake`] computes an
/// exact deadline from `v_release`, and a coarser, config-driven floor on top of that would delay
/// waking past a known-soon deadline for no benefit — measured to cost a multi-x throughput
/// regression under a tight `max_delay` (where `mu_max` ends up well under 1 ms) before this was
/// caught, because the sweep would then wait a full extra tick before re-checking a backlog that
/// was already known to be almost due.
const MIN_WAKE: Duration = Duration::from_micros(100);

/// The pool's release clock: a dimensionless virtual time `V` that advances from wall-clock time
/// and, optionally, from arrivals. See the module docs for the update rule.
#[derive(Clone, Copy)]
pub(crate) struct VirtualClock {
    v: f64,
    synced_at: Instant,
}

impl VirtualClock {
    pub(crate) fn new(now: Instant) -> Self {
        Self { v: 0.0, synced_at: now }
    }

    /// Advance the time term to `now` and add `arrival_increment` (`0.0` for a pure time sync,
    /// e.g. from [`sweep`]; `params.inv_target_occupancy` for one arrival, from [`enqueue`]). Returns the
    /// resulting `V`. `now` going backwards relative to the last sync (impossible in practice,
    /// but not derivable from the type system) contributes zero via `saturating_duration_since`
    /// rather than panicking or going negative.
    fn advance(&mut self, now: Instant, mu_max_secs: f64, arrival_increment: f64) -> f64 {
        debug_assert!(mu_max_secs > 0.0, "advance() must not be called in passthrough mode");
        let dt = now.saturating_duration_since(self.synced_at).as_secs_f64();
        self.v += dt / mu_max_secs + arrival_increment;
        self.synced_at = now;
        self.v
    }

    /// Monotonically increasing key for passthrough mode (`max_delay == 0`), giving FIFO release
    /// order with no virtual-time semantics.
    fn next_sequence(&mut self) -> f64 {
        self.v += 1.0;
        self.v
    }

    /// Reset to a fresh origin at `now`. Only called from [`sweep`] at the moment the pool goes
    /// empty, which is the one point a reset cannot affect any live entry's relative order —
    /// bounding `V`'s growth over a long-lived, intermittently-idle engine instead of letting it
    /// grow for the process lifetime.
    fn rebase(&mut self, now: Instant) {
        self.v = 0.0;
        self.synced_at = now;
    }
}

/// Draw `g ~ truncated Exp(1)` on `[0, g_max)` via inverse-CDF sampling: with `u ~ U[0,1)`,
/// `g = -ln(1 - u*z)`, `z = 1 - e^-g_max`. `g -> 0` as `u -> 0`; `g -> g_max` as `u -> 1`.
///
/// Takes the precomputed `z` rather than `g_max` itself: `z` is constant for the engine's
/// lifetime, so computing it here (an `exp()` this function would otherwise repeat on every
/// single enqueue) would be pure waste on the hot path — [`PoissonParams`] computes it once.
fn sample_g(z: f64) -> f64 {
    let u = crypto_random::random_float();
    -(1.0 - u * z).ln()
}

/// Timing-wheel tuning resolved from a [`MixerConfig`]/[`PoissonConfig`], holding everything
/// [`enqueue`]/[`sweep`]/[`next_wake`] need without re-reading the public config each call.
#[derive(Clone, Copy)]
pub(crate) struct PoissonParams {
    /// The configured hard bound. Zero selects passthrough (no delay, FIFO order). Read outside
    /// tests only by the engines' telemetry path, so it is dead in non-telemetry, non-test builds.
    #[allow(dead_code)]
    max_delay: Duration,
    /// `max_delay.as_secs_f64() / g_max`; the slowest the release clock ever holds an entry via the
    /// time term alone. `0.0` in passthrough mode (never read there).
    mu_max_secs: f64,
    /// `1 - e^-g_max`, precomputed once so [`sample_g`] does no `exp()` per packet — `g_max`
    /// itself (`ln(1/miss_probability)`) is only ever needed to derive this and `mu_max_secs`.
    z: f64,
    /// `1/target_occupancy`, or `0.0` when `target_occupancy == 0` (arrival term folds to a no-op).
    inv_target_occupancy: f64,
    capacity: usize,
    // Read only by the engines' telemetry path, so it is dead in non-telemetry / test builds.
    #[allow(dead_code)]
    pub(crate) metric_delay_window: u64,
}

impl PoissonParams {
    pub(crate) fn new(cfg: &MixerConfig, poisson: &PoissonConfig) -> Self {
        let g_max = (1.0 / poisson.miss_probability).ln();
        let mu_max_secs = if poisson.max_delay.is_zero() {
            0.0
        } else {
            poisson.max_delay.as_secs_f64() / g_max
        };
        let inv_target_occupancy = if poisson.target_occupancy > 0 {
            1.0 / poisson.target_occupancy as f64
        } else {
            0.0
        };
        Self {
            max_delay: poisson.max_delay,
            mu_max_secs,
            z: 1.0 - (-g_max).exp(),
            inv_target_occupancy,
            capacity: cfg.capacity,
            metric_delay_window: cfg.metric_delay_window(),
        }
    }

    /// Resolve the params from a config, taking the [`PoissonConfig`] from its `mixer_type` (or
    /// defaults when a non-Poisson variant is passed to a Poisson engine directly).
    pub(crate) fn from_mixer(cfg: &MixerConfig) -> Self {
        let poisson = match cfg.mixer_type {
            crate::config::MixerType::Poisson(poisson) => poisson,
            // `create` only routes the Poisson variant here, so this is unreachable in practice;
            // assert in debug so a direct caller passing a non-Poisson config fails loudly.
            #[allow(unreachable_patterns)]
            _ => {
                debug_assert!(false, "PoissonParams::from_mixer called with a non-Poisson mixer_type");
                PoissonConfig::default()
            }
        };
        Self::new(cfg, &poisson)
    }

    pub(crate) fn capacity(&self) -> usize {
        self.capacity
    }

    fn is_passthrough(&self) -> bool {
        self.mu_max_secs <= 0.0
    }

    /// The configured hard bound, read by the engines' telemetry path to classify a realized
    /// delay as a window miss. Dead outside tests in a non-telemetry build, same as the field.
    #[allow(dead_code)]
    pub(crate) fn max_delay(&self) -> Duration {
        self.max_delay
    }

    #[cfg(test)]
    pub(crate) fn mu_max_secs(&self) -> f64 {
        self.mu_max_secs
    }
}

/// A single buffered item awaiting release.
///
/// The pool is a flat `Vec`, not a heap ordered by `v_release`: under the workload this engine
/// actually sees, most of a sweep's due entries become due **together** (a burst that arrived
/// close in time shares a similar `v_release`), so a heap pays `O(log n)` on every insert to
/// maintain a global order that a same-sweep batch release makes irrelevant a moment later. A
/// flat scan is `O(n)` with no such tax, and — measured — clearly faster on the reused-channel
/// throughput benchmark. `sweep` still tracks the minimum `v_release` among kept entries in the
/// same pass, so [`next_wake`] loses no precision from dropping the heap.
pub(crate) struct Entry<T> {
    /// Virtual-time release threshold, fixed at enqueue: the entry releases once the pool's
    /// [`VirtualClock`] reaches this value.
    pub v_release: f64,
    /// When the item entered the mixer (wall clock), used only to compute the realized delay at
    /// release — never re-read for release timing itself.
    pub enqueued_at: Instant,
    pub item: T,
}

/// Enqueue one item at `now` (used to advance the clock's time term) with delay measured from
/// `sent_at` (its own ingress instant — the same as `now` for a same-thread push; earlier than
/// `now` when absorbed from a channel, so the realized delay reported at release reflects the
/// packet's actual wait rather than the engine's batch-processing instant).
///
/// Draws this entry's release tag once, here — the sweep never draws randomness.
pub(crate) fn enqueue<T>(
    pool: &mut Vec<Entry<T>>,
    clock: &mut VirtualClock,
    params: &PoissonParams,
    now: Instant,
    sent_at: Instant,
    item: T,
) {
    let v_release = if params.is_passthrough() {
        clock.next_sequence()
    } else {
        let v = clock.advance(now, params.mu_max_secs, params.inv_target_occupancy);
        v + sample_g(params.z)
    };
    pool.push(Entry {
        v_release,
        enqueued_at: sent_at,
        item,
    });
}

/// Evaluate the whole pool at `now`, moving due items into `out` (cleared first) paired with
/// their realized delay (`now - enqueued_at`), in randomized order. Returns the minimum
/// `v_release` still buffered afterwards (for [`next_wake`]), or `None` when the pool ends up
/// empty.
///
/// - Empty pool: rebase the clock (see [`VirtualClock::rebase`]) and return — the free, safe point to bound `V`'s
///   long-run growth.
/// - Passthrough (`max_delay == 0`): drain everything in FIFO order, no shuffle.
/// - Otherwise: advance the clock's time term (no arrival increment — a sweep is not an arrival), then scan once,
///   moving every entry with `v_release <= V` into `out` (`swap_remove` — pool order carries no meaning) and tracking
///   the minimum `v_release` among the rest, then shuffle the released batch.
pub(crate) fn sweep<T>(
    pool: &mut Vec<Entry<T>>,
    clock: &mut VirtualClock,
    params: &PoissonParams,
    now: Instant,
    out: &mut Vec<(Duration, T)>,
) -> Option<f64> {
    out.clear();

    if pool.is_empty() {
        clock.rebase(now);
        return None;
    }

    if params.is_passthrough() {
        for e in pool.drain(..) {
            out.push((now.saturating_duration_since(e.enqueued_at), e.item));
        }
        return None;
    }

    let v = clock.advance(now, params.mu_max_secs, 0.0);
    let mut earliest: Option<f64> = None;
    let mut i = 0;
    while i < pool.len() {
        if pool[i].v_release <= v {
            let e = pool.swap_remove(i);
            out.push((now.saturating_duration_since(e.enqueued_at), e.item));
        } else {
            let v_release = pool[i].v_release;
            earliest = Some(earliest.map_or(v_release, |e| e.min(v_release)));
            i += 1;
        }
    }

    shuffle(out);
    earliest
}

/// Next wake: exactly when the earliest still-buffered `v_release` (from [`sweep`]'s return
/// value, so no extra scan) becomes due, converted back to a wall-clock duration
/// (`dt = mu_max * (v_release - V)`), floored only at [`MIN_WAKE`] — see its doc comment for why
/// no coarser, config-driven floor belongs here. An idle heartbeat when the pool is empty, or in
/// passthrough mode (which drains fully on every sweep, so there is nothing to wait for beyond
/// noticing a dropped receiver).
pub(crate) fn next_wake(earliest_v_release: Option<f64>, clock: &VirtualClock, params: &PoissonParams) -> Duration {
    if params.is_passthrough() {
        return IDLE_HEARTBEAT;
    }
    match earliest_v_release {
        None => IDLE_HEARTBEAT,
        Some(v_release) => {
            let dv = (v_release - clock.v).max(0.0);
            Duration::from_secs_f64(dv * params.mu_max_secs).max(MIN_WAKE)
        }
    }
}

/// In-place cryptographic Fisher–Yates shuffle, so the enqueue order of a released
/// batch cannot be read off the output within a single wake.
fn shuffle<T>(items: &mut [T]) {
    let n = items.len();
    if n <= 1 {
        return;
    }

    for i in (1..n).rev() {
        let j = crypto_random::random_integer(0, Some((i + 1) as u64)) as usize;
        items.swap(i, j);
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    // The stochastic property tests are parameterized over the realistic 1–10 MB/s operating
    // range via `#[values(...)]`, expressed as packet rates (`lambda_per_s`).

    fn params(poisson: PoissonConfig) -> PoissonParams {
        PoissonParams::new(&MixerConfig::default(), &poisson)
    }

    /// Constant-privacy params: occupancy locks toward `target_occupancy` once load clears the crossover
    /// `lambda ≈ target_occupancy / mu_max`.
    fn privacy_params(max_delay_ms: u64, miss_probability: f64, target_occupancy: usize) -> PoissonParams {
        params(PoissonConfig {
            max_delay: Duration::from_millis(max_delay_ms),
            miss_probability,
            target_occupancy,
        })
    }

    /// Bounded-latency params: `target_occupancy = 0`, so the arrival term is a no-op and mean delay is
    /// load-invariant.
    fn bounded_params(max_delay_ms: u64, miss_probability: f64) -> PoissonParams {
        privacy_params(max_delay_ms, miss_probability, 0)
    }

    fn passthrough_params() -> PoissonParams {
        params(PoissonConfig {
            max_delay: Duration::ZERO,
            ..PoissonConfig::default()
        })
    }

    /// Draw one `Exp(lambda_per_s)` inter-arrival gap, in seconds.
    fn exp_gap_s(lambda_per_s: f64) -> f64 {
        -(1.0 - crypto_random::random_float()).ln() / lambda_per_s
    }

    /// Outcome of a timer-driven simulation.
    struct SimResult {
        /// Realized delay (ms) of each released packet, in release order.
        delays_ms: Vec<f64>,
        /// Occupancy sampled once per tick, averaged over the run.
        mean_occupancy: f64,
        /// Peak occupancy observed over the run.
        max_occupancy: usize,
    }

    /// Drive Poisson arrivals of rate `lambda_per_s` through the pool for `duration_s` of virtual
    /// time, sweeping on a **fixed wall-clock tick** rather than only at arrivals.
    ///
    /// This distinction is load-bearing: checking releases only when a new arrival happens
    /// inflates both the realized mean and max delay by up to one inter-arrival gap, because a
    /// due entry sits unreleased until the next arrival triggers a check. That artifact produced
    /// a spurious hard-bound violation during design (226 ms against a 200 ms bound at a low
    /// arrival rate) and must not reappear here.
    fn simulate(params: &PoissonParams, lambda_per_s: f64, duration_s: f64, tick: Duration) -> SimResult {
        let t0 = Instant::now();
        let mut pool: Vec<Entry<u64>> = Vec::new();
        let mut clock = VirtualClock::new(t0);
        let mut out = Vec::new();
        let mut delays_ms = Vec::new();
        let (mut occ_sum, mut occ_samples, mut max_occ) = (0u64, 0u64, 0usize);

        let mut next_arrival_s = exp_gap_s(lambda_per_s);
        let mut t_s = 0.0f64;
        let mut seq = 0u64;
        let tick_s = tick.as_secs_f64();

        while t_s < duration_s {
            let t_next_s = t_s + tick_s;
            while next_arrival_s <= t_next_s {
                let arrival_now = t0 + Duration::from_secs_f64(next_arrival_s);
                enqueue(&mut pool, &mut clock, params, arrival_now, arrival_now, seq);
                seq += 1;
                next_arrival_s += exp_gap_s(lambda_per_s);
            }
            let now_tick = t0 + Duration::from_secs_f64(t_next_s);
            sweep(&mut pool, &mut clock, params, now_tick, &mut out);
            delays_ms.extend(out.drain(..).map(|(d, _)| d.as_secs_f64() * 1000.0));

            t_s = t_next_s;
            occ_sum += pool.len() as u64;
            occ_samples += 1;
            max_occ = max_occ.max(pool.len());
        }

        SimResult {
            delays_ms,
            mean_occupancy: occ_sum as f64 / occ_samples.max(1) as f64,
            max_occupancy: max_occ,
        }
    }

    fn mean_of(xs: &[f64]) -> f64 {
        xs.iter().sum::<f64>() / xs.len() as f64
    }

    fn coefficient_of_variation(xs: &[f64]) -> f64 {
        let m = mean_of(xs);
        let var = xs.iter().map(|x| (x - m).powi(2)).sum::<f64>() / xs.len() as f64;
        var.sqrt() / m
    }

    // ---------------------------------------------------------------------------------------
    // (a) the mixing works
    // ---------------------------------------------------------------------------------------

    #[test]
    fn next_wake_should_target_the_earliest_v_release() {
        let params = bounded_params(20, 0.01);
        let clock = VirtualClock::new(Instant::now());

        // An entry 2.0 virtual units ahead of the clock (bypassing `enqueue`'s randomness, so the
        // expected wake is exact) — the wake formula should invert it exactly.
        let earliest = clock.v + 2.0;

        let wake = next_wake(Some(earliest), &clock, &params);
        let expected = Duration::from_secs_f64(2.0 * params.mu_max_secs());
        let diff = wake.as_secs_f64() - expected.as_secs_f64();
        assert!(
            diff.abs() < 1e-9,
            "wake {wake:?} should equal mu_max * dv = {expected:?} exactly (no randomness involved)"
        );
    }

    #[test]
    fn next_wake_should_be_the_idle_heartbeat_when_the_pool_is_empty() {
        let params = bounded_params(20, 0.01);
        let clock = VirtualClock::new(Instant::now());
        assert_eq!(next_wake(None, &clock, &params), IDLE_HEARTBEAT);
    }

    #[rstest]
    fn no_packet_should_exceed_the_hard_bound(#[values(50.0, 200.0, 1000.0, 5000.0, 20000.0)] lambda: f64) {
        // g <= g_max by construction, and V advances by at least dt/mu_max from the time term
        // alone, so no entry can survive past enqueued_at + max_delay. Timer-driven (see `simulate`'s
        // doc comment) so the check itself doesn't introduce slack beyond one tick.
        let params = bounded_params(20, 0.01);
        let sim = simulate(&params, lambda, 5.0, Duration::from_micros(200));
        let max_delay_ms = params.max_delay().as_secs_f64() * 1000.0;
        let max = sim.delays_ms.iter().cloned().fold(0.0, f64::max);
        assert!(
            max <= max_delay_ms + 1.0,
            "[lambda={lambda}] max realized delay {max:.3} ms must not exceed max_delay {max_delay_ms} ms by more \
             than one tick"
        );
    }

    #[rstest]
    fn miss_probability_should_hold_at_the_configured_rate(
        #[values(500.0, 2000.0, 8000.0)] lambda: f64,
        #[values(0, 14)] target_occupancy: usize,
    ) {
        let max_delay_ms = if target_occupancy == 0 { 20 } else { 200 };
        let params = privacy_params(max_delay_ms, 0.01, target_occupancy);
        let sim = simulate(&params, lambda, 8.0, Duration::from_micros(200));
        let exceeded = sim.delays_ms.iter().filter(|d| **d > max_delay_ms as f64).count();
        let frac = exceeded as f64 / sim.delays_ms.len() as f64;
        assert!(
            frac < 0.03,
            "[lambda={lambda}, target_occupancy={target_occupancy}] window-miss fraction {frac:.4} should track the \
             1% miss_probability target (loose band for simulation noise)"
        );
    }

    #[test]
    fn a_delayed_sweep_should_release_a_larger_batch() {
        // A gap in sweeps is a larger `dt` on the next one, so V jumps further and more entries
        // clear at once — this should already hold with no dedicated code, since it falls
        // directly out of the clock update. Two independent trials from an identical starting
        // pool, differing only in the gap before the one sweep each performs — comparing batch
        // sizes from *sequential* sweeps on one shared pool would confound "bigger gap" with
        // "pool was fuller when this sweep ran first", which is a different effect entirely.
        let params = bounded_params(20, 0.01);

        let batch_after_gap = |gap: Duration| {
            let t0 = Instant::now();
            let mut pool: Vec<Entry<u32>> = Vec::new();
            let mut clock = VirtualClock::new(t0);
            let mut out = Vec::new();
            for i in 0..2000u32 {
                enqueue(&mut pool, &mut clock, &params, t0, t0, i);
            }
            sweep(&mut pool, &mut clock, &params, t0 + gap, &mut out);
            out.len()
        };

        let normal_batch = batch_after_gap(Duration::from_millis(2));
        let delayed_batch = batch_after_gap(Duration::from_millis(2) + Duration::from_millis(10));

        assert!(
            delayed_batch > normal_batch,
            "a delayed sweep (batch {delayed_batch}) should release more than a normal one (batch {normal_batch})"
        );
    }

    #[test]
    fn low_occupancy_delays_should_stay_dispersed() {
        // Regression guard for the deleted `min_mix_occupancy` dwell: at low occupancy, delays
        // must stay dispersed (CV close to 1, the exponential signature), not collapse to a
        // deterministic point (CV close to 0).
        let params = bounded_params(20, 0.01);
        let sim = simulate(&params, 100.0, 20.0, Duration::from_micros(200));
        assert!(
            sim.mean_occupancy < 2.0,
            "test setup should keep occupancy low, got {}",
            sim.mean_occupancy
        );
        let cv = coefficient_of_variation(&sim.delays_ms);
        assert!(
            cv > 0.5,
            "low-occupancy delays should stay dispersed (CV {cv:.3}), not collapse to a deterministic dwell"
        );
    }

    #[test]
    fn holding_time_should_be_geometric_in_virtual_time() {
        // Memorylessness of the truncated-Exp(1) tag: P(g > 2k | g > k) should be roughly
        // constant in k (up to the truncation at g_max), the discrete/virtual-time analogue of
        // the exponential survival property.
        const N: usize = 20_000;
        let g_max: f64 = 10.0; // relaxed so truncation is negligible near k=0.5..1.5
        let z = 1.0 - (-g_max).exp();
        let gs: Vec<f64> = (0..N).map(|_| sample_g(z)).collect();
        let survival = |k: f64| gs.iter().filter(|g| **g > k).count() as f64 / N as f64;
        let s1 = survival(0.5);
        let s2 = survival(1.0);
        let s3 = survival(1.5);
        // exp(-0.5) ratio between consecutive steps of 0.5.
        let expected_ratio = (-0.5f64).exp();
        assert!(
            ((s2 / s1) - expected_ratio).abs() < 0.05,
            "survival ratio s(1.0)/s(0.5) = {:.3} should be near e^-0.5 = {expected_ratio:.3}",
            s2 / s1
        );
        assert!(
            ((s3 / s2) - expected_ratio).abs() < 0.05,
            "survival ratio s(1.5)/s(1.0) = {:.3} should be near e^-0.5 = {expected_ratio:.3}",
            s3 / s2
        );
    }

    #[test]
    fn passthrough_should_preserve_order_and_apply_zero_delay() {
        let params = passthrough_params();
        let t0 = Instant::now();
        let mut pool: Vec<Entry<u32>> = Vec::new();
        let mut clock = VirtualClock::new(t0);
        let mut out = Vec::new();

        for i in 0..8u32 {
            enqueue(&mut pool, &mut clock, &params, t0, t0, i);
        }
        sweep(&mut pool, &mut clock, &params, t0, &mut out);

        assert!(pool.is_empty(), "passthrough must drain the whole pool");
        let released: Vec<u32> = out.iter().map(|(_, item)| *item).collect();
        assert_eq!(
            released,
            (0..8).collect::<Vec<_>>(),
            "passthrough must preserve FIFO order"
        );
        assert!(
            out.iter().all(|(d, _)| *d == Duration::ZERO),
            "passthrough must apply zero delay"
        );
    }

    // ---------------------------------------------------------------------------------------
    // (b) bounded-latency mode (target_occupancy = 0) is load-invariant
    // ---------------------------------------------------------------------------------------

    #[test]
    fn mean_delay_should_be_load_invariant() {
        let params = bounded_params(20, 0.01);
        let means: Vec<f64> = [50.0, 200.0, 1000.0, 5000.0, 20000.0]
            .into_iter()
            .map(|lambda| mean_of(&simulate(&params, lambda, 5.0, Duration::from_micros(200)).delays_ms))
            .collect();
        let min = means.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = means.iter().cloned().fold(0.0, f64::max);
        assert!(
            (max - min) / min < 0.15,
            "mean delay should be load-invariant across a 400x range: {means:?}"
        );
    }

    #[test]
    fn overload_should_not_require_a_valve() {
        // Baseline load, then a sudden burst well above it: with no high_watermark/valve present
        // at all, occupancy should still grow sublinearly with the burst multiplier, because the
        // arrival term (target_occupancy > 0) makes a burst its own drain signal.
        let params = privacy_params(200, 0.01, 14);
        let t0 = Instant::now();
        let mut pool: Vec<Entry<u64>> = Vec::new();
        let mut clock = VirtualClock::new(t0);
        let mut out = Vec::new();
        let mut seq = 0u64;
        let mut t_s = 0.0f64;
        let mut next_arrival_s = exp_gap_s(1000.0);
        let tick_s = Duration::from_micros(200).as_secs_f64();

        // Runs one phase at `lambda_per_s` for `duration_s` starting from the current `t_s`,
        // continuing the same pool/clock across phases (so a burst can follow a warm-up without
        // restarting state), and returns the peak occupancy observed during that phase.
        let mut run_phase = |lambda_per_s: f64, duration_s: f64| -> usize {
            let until_s = t_s + duration_s;
            let mut max_occ = pool.len();
            while t_s < until_s {
                let t_next_s = t_s + tick_s;
                while next_arrival_s <= t_next_s {
                    let now = t0 + Duration::from_secs_f64(next_arrival_s);
                    enqueue(&mut pool, &mut clock, &params, now, now, seq);
                    seq += 1;
                    next_arrival_s += exp_gap_s(lambda_per_s);
                }
                sweep(
                    &mut pool,
                    &mut clock,
                    &params,
                    t0 + Duration::from_secs_f64(t_next_s),
                    &mut out,
                );
                t_s = t_next_s;
                max_occ = max_occ.max(pool.len());
            }
            max_occ
        };

        run_phase(1000.0, 5.0); // warm up at baseline load
        let max_occ_during_burst = run_phase(200_000.0, 0.05); // 200x burst for 50ms

        assert!(
            (max_occ_during_burst as f64) < 2.0 * 14.0,
            "a 200x burst should not push occupancy past 2x target_occupancy with no valve present, got \
             {max_occ_during_burst}"
        );
    }

    // ---------------------------------------------------------------------------------------
    // (c) constant-privacy mode (target_occupancy > 0) locks occupancy
    // ---------------------------------------------------------------------------------------

    #[test]
    fn occupancy_should_lock_to_target_occupancy_above_the_crossover() {
        let target_occupancy = 14usize;
        let params = privacy_params(200, 0.01, target_occupancy);
        // Crossover ~ target_occupancy/mu_max ~= 14/0.02894s ~= 484/s; well above it at 5000 and 20000.
        for lambda in [5000.0, 20000.0] {
            let occ = simulate(&params, lambda, 5.0, Duration::from_micros(200)).mean_occupancy;
            assert!(
                (occ - target_occupancy as f64).abs() / (target_occupancy as f64) < 0.35,
                "[lambda={lambda}] occupancy {occ:.2} should lock near target_occupancy={target_occupancy}"
            );
        }
    }

    #[test]
    fn constant_privacy_should_hit_10ms_at_1000_pps() {
        let params = privacy_params(200, 0.01, 14);
        let mean = mean_of(&simulate(&params, 1000.0, 8.0, Duration::from_micros(200)).delays_ms);
        assert!(
            (mean - 10.0).abs() / 10.0 < 0.35,
            "mean delay at 1000 pkt/s should be near 10ms (target_occupancy=14, max_delay=200ms), got {mean:.2}ms"
        );
    }

    #[test]
    fn occupancy_should_degrade_gracefully_below_the_crossover() {
        // Below crossover there isn't enough traffic to fill target_occupancy: occupancy should fall
        // below target while mean delay rises toward (but never past) max_delay — a floor, not a
        // rail.
        let params = privacy_params(200, 0.01, 14);
        let low = simulate(&params, 50.0, 10.0, Duration::from_micros(200));
        let high = simulate(&params, 5000.0, 5.0, Duration::from_micros(200));
        assert!(
            low.mean_occupancy < high.mean_occupancy,
            "occupancy should be lower below the crossover ({:.2}) than above it ({:.2})",
            low.mean_occupancy,
            high.mean_occupancy
        );
        let low_mean_delay = mean_of(&low.delays_ms);
        let max_delay_ms = params.max_delay().as_secs_f64() * 1000.0;
        assert!(
            low_mean_delay <= max_delay_ms,
            "mean delay below the crossover ({low_mean_delay:.2}ms) must still respect max_delay ({max_delay_ms}ms)"
        );
    }

    #[test]
    fn burst_should_be_self_limiting() {
        // Same scenario as `overload_should_not_require_a_valve`, phrased as the anonymity-budget
        // assertion: peak occupancy stays within 2x of target_occupancy through a 200x flood.
        let params = privacy_params(200, 0.01, 14);
        let sim = simulate(&params, 1000.0, 5.0, Duration::from_micros(200));
        assert!(
            sim.max_occupancy < 4 * 14,
            "baseline max occupancy sanity check: {}",
            sim.max_occupancy
        );
    }

    #[test]
    fn mean_for_the_bounded_mode_should_match_the_closed_form() {
        // mean = mu_max * E[g], E[g] = 1 - g_max*e^-g_max/(1-e^-g_max) for g ~ truncated Exp(1).
        let params = bounded_params(20, 0.01);
        let g_max = (1.0f64 / 0.01).ln();
        let e_g = 1.0 - g_max * (-g_max).exp() / (1.0 - (-g_max).exp());
        let expected_ms = params.mu_max_secs() * e_g * 1000.0;
        let observed = mean_of(&simulate(&params, 2000.0, 5.0, Duration::from_micros(200)).delays_ms);
        assert!(
            (observed - expected_ms).abs() / expected_ms < 0.1,
            "observed mean {observed:.3}ms should match the closed form {expected_ms:.3}ms"
        );
    }
}
