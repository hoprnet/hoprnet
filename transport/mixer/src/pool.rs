//! Pure buffer-and-sweep logic for the exponential (Poisson) release engine.
//!
//! This module is deliberately free of any async, threading or channel concerns so
//! that the release decision can be unit-tested in isolation. The engine in
//! [`crate::poisson`] owns a `Vec<Entry<T>>` and calls [`sweep`] on every wake.

use std::time::{Duration, Instant};

use hopr_types::crypto_random;

use crate::config::{HOPR_MIXER_CAP_PERCENTILE, MixerConfig, PoissonConfig};

/// Idle heartbeat when the pool is empty, so an engine periodically notices a dropped receiver
/// and can shut down instead of parking forever. Both engines use it as the initial timer.
pub(crate) const IDLE_HEARTBEAT: Duration = Duration::from_millis(200);
const MIN_WAKE: Duration = Duration::from_micros(100);

/// Poisson tuning resolved from a [`MixerConfig`], holding everything the sweep and wake policy
/// need without re-reading the public config or recomputing the derived mean each tick.
#[derive(Clone, Copy)]
pub(crate) struct PoissonParams {
    min_delay: Duration,
    cap: Duration,
    cap_jitter: Duration,
    min_mix_occupancy: usize,
    high_watermark: usize,
    capacity: usize,
    mean: Duration,
    saturation_min_mean: Duration,
    tick_floor: Duration,
    // Read only by the engines' telemetry path, so it is dead in non-telemetry / test builds.
    #[allow(dead_code)]
    pub(crate) metric_delay_window: u64,
}

impl PoissonParams {
    pub(crate) fn new(cfg: &MixerConfig, poisson: &PoissonConfig) -> Self {
        Self {
            min_delay: cfg.min_delay,
            cap: cfg.cap(),
            cap_jitter: poisson.cap_jitter,
            min_mix_occupancy: poisson.min_mix_occupancy,
            high_watermark: poisson.high_watermark,
            capacity: cfg.capacity,
            mean: derive_mean(cfg.delay_range, poisson.target_mean_delay),
            saturation_min_mean: poisson.saturation_min_mean,
            tick_floor: poisson.tick_floor,
            metric_delay_window: cfg.metric_delay_window,
        }
    }

    /// Resolve the params from a config, taking the [`PoissonConfig`] from its `mixer_type` (or
    /// defaults when a non-Poisson variant is passed to a Poisson engine directly).
    pub(crate) fn from_mixer(cfg: &MixerConfig) -> Self {
        let poisson = match cfg.mixer_type {
            #[cfg(feature = "poisson")]
            crate::config::MixerType::Poisson(poisson) => poisson,
            #[cfg(feature = "poisson-shared")]
            crate::config::MixerType::PoissonShared(poisson) => poisson,
            // `create` only routes Poisson variants here, so this is unreachable in practice;
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
        self.cap.is_zero()
    }

    /// Effective mean under the overload valve: the base mean up to `high_watermark`, shrinking
    /// toward `saturation_min_mean` as occupancy nears `capacity`. The watermark is clamped below
    /// `capacity` so a small configured capacity can't push the trip point out of reach.
    fn mean_for(&self, occupancy: usize) -> Duration {
        let base = self.mean;
        let watermark = self.high_watermark.min(self.capacity.saturating_sub(1));
        if occupancy <= watermark {
            return base;
        }
        let low = watermark.max(1);
        let high = self.capacity.max(low + 1);
        let fraction = (occupancy.saturating_sub(low) as f64 / (high - low) as f64).clamp(0.0, 1.0);
        let scaled = Duration::from_secs_f64(base.as_secs_f64() * (1.0 - fraction));
        scaled.max(self.saturation_min_mean.min(base))
    }

    /// Adaptive wake interval `mean / occupancy`, clamped to `[tick_floor, mean]`.
    fn adaptive_interval(&self, occupancy: usize) -> Duration {
        let ceil = self.mean.max(self.tick_floor);
        Duration::from_secs_f64(self.mean.as_secs_f64() / occupancy.max(1) as f64).clamp(self.tick_floor, ceil)
    }
}

/// Memoryless release probability `1 - e^(-delta/mean)`, independent of the wake cadence.
fn release_probability(delta: Duration, mean: Duration) -> f64 {
    let mean = mean.as_secs_f64();
    if mean <= 0.0 {
        return 1.0;
    }
    1.0 - (-delta.as_secs_f64() / mean).exp()
}

/// Explicit `target` when set, else derived from `delay_range` so [`HOPR_MIXER_CAP_PERCENTILE`]
/// of packets release before the cap. The window is `delay_range` because the clock only runs
/// past `min_delay`.
fn derive_mean(delay_range: Duration, target: Duration) -> Duration {
    if !target.is_zero() {
        return target;
    }
    let window = delay_range.as_secs_f64();
    let factor = (1.0 / (1.0 - HOPR_MIXER_CAP_PERCENTILE)).ln();
    if window <= 0.0 || factor <= 0.0 {
        return Duration::ZERO;
    }
    Duration::from_secs_f64(window / factor)
}

/// Next wake from the current occupancy and the earliest still-buffered enqueue instant (from
/// [`sweep`], so no extra scan): the adaptive interval, capped by the soonest jitter-window
/// opening, floored at `MIN_WAKE`; an idle heartbeat when the pool is empty.
pub(crate) fn next_wake(
    earliest_enqueued: Option<Instant>,
    occupancy: usize,
    params: &PoissonParams,
    now: Instant,
) -> Duration {
    match earliest_enqueued {
        None => IDLE_HEARTBEAT,
        Some(enqueued_at) => {
            let interval = params.adaptive_interval(occupancy);
            let window_opens = params.cap.saturating_sub(params.cap_jitter);
            let deadline_wake = (enqueued_at + window_opens).saturating_duration_since(now);
            interval.min(deadline_wake).max(MIN_WAKE)
        }
    }
}

/// A single buffered item awaiting release.
pub(crate) struct Entry<T> {
    /// When the item entered the mixer.
    pub enqueued_at: Instant,
    /// Per-entry uniform sample in `[0, 1)`, drawn **once** at enqueue, that places this entry's
    /// hard-cap force-release deterministically within `[cap - cap_jitter, cap)`. Sampling once
    /// here (rather than re-drawing each sweep) keeps the smear uniform over the window and
    /// independent of the wake cadence — re-drawing per sweep would take the minimum over many
    /// draws, concentrating releases near `cap - cap_jitter` and coupling them to the tick rate.
    pub jitter_fraction: f64,
    pub item: T,
}

impl<T> Entry<T> {
    pub(crate) fn new(enqueued_at: Instant, item: T) -> Self {
        Self {
            enqueued_at,
            jitter_fraction: crypto_random::random_float(),
            item,
        }
    }
}

/// Evaluate the whole pool at `now`, moving items to release into `out` (cleared first) paired
/// with their realized delay (`now - enqueued_at`), in randomized order. `delta` is the elapsed
/// time since the previous sweep — the amount the memoryless clock advances this tick. Returns
/// the earliest `enqueued_at` still buffered (for deadline-aware wake scheduling), or `None`
/// when the pool is empty afterwards.
///
/// Release rules per item (first match wins):
/// 1. `age < min_delay` — not yet eligible, keep.
/// 2. `age >= cap_deadline` — hard-cap force-release, where `cap_deadline` is this entry's deadline within `[cap -
///    cap_jitter, cap)`, fixed once at enqueue (equals `cap` when jitter is zero).
/// 3. `occupancy <= min_mix_occupancy` — small buffer: deterministic minimum dwell, released once `age >= mean` (no
///    coin), so the few packets present overlap and mix instead of escaping through the exponential's fast tail.
/// 4. `Bernoulli(1 - e^(-delta/mean))` — the memoryless exponential clock.
///
/// When no delay is configured (`is_passthrough`) the pool is drained in enqueue order without
/// mixing, preserving FIFO semantics.
///
/// Performance: for the common case (a packet buffered since before the previous sweep) the
/// release probability depends only on `delta` — shared by the whole pool this tick — so
/// `1 - e^(-delta/mean)` is computed **once** and reused. Only a packet that arrived mid-interval
/// (eligible for less than `delta`) needs its own `1 - e^(-eligible_for/mean)`. The hard-cap
/// deadline is precomputed per entry at enqueue, so the sweep itself does no jitter sampling.
pub(crate) fn sweep<T>(
    pool: &mut Vec<Entry<T>>,
    params: &PoissonParams,
    now: Instant,
    delta: Duration,
    out: &mut Vec<(Duration, T)>,
) -> Option<Instant> {
    out.clear();
    if pool.is_empty() {
        return None;
    }

    if params.is_passthrough() {
        for e in pool.drain(..) {
            out.push((now.saturating_duration_since(e.enqueued_at), e.item));
        }
        return None;
    }

    let occupancy = pool.len();
    let cap = params.cap;
    let jitter = params.cap_jitter;
    let min_delay = params.min_delay;
    let min_mix_occupancy = params.min_mix_occupancy;
    let mean = params.mean_for(occupancy);

    // Computed once per sweep (memoryless, shared `delta`) rather than per packet.
    let full_delta_probability = release_probability(delta, mean);
    let jitter_window_start = cap.saturating_sub(jitter);

    let mut earliest: Option<Instant> = None;
    let mut i = 0;
    while i < pool.len() {
        let enqueued_at = pool[i].enqueued_at;
        let age = now.saturating_duration_since(enqueued_at);
        // This entry's hard-cap deadline, fixed at enqueue: `[cap - jitter, cap)`, or exactly
        // `cap` when jitter is zero. Uniform over the window and independent of the sweep cadence.
        let cap_deadline = jitter_window_start + jitter.mul_f64(pool[i].jitter_fraction);

        let release = if age < min_delay {
            false
        } else if age >= cap_deadline {
            true // hard-cap force-release
        } else if occupancy <= min_mix_occupancy {
            age >= mean // small buffer: deterministic minimum dwell
        } else {
            // Memoryless coin. The clock advances only over the time the packet has actually been
            // eligible: a packet present since before the last sweep gets the full `delta` (the
            // precomputed probability — the common case), but one that arrived mid-interval only
            // gets `age - min_delay`, so a large `delta` (e.g. after an idle gap) can't over-release
            // a fresh burst.
            let eligible_for = age.saturating_sub(min_delay);
            let p = if eligible_for >= delta {
                full_delta_probability
            } else {
                release_probability(eligible_for, mean)
            };
            crypto_random::random_float() < p
        };

        if release {
            // `swap_remove` is O(1); pool order carries no meaning (the released
            // batch is shuffled below and the kept items are re-evaluated next sweep).
            // `age` is `now - enqueued_at` of this very entry (computed above), so reuse it as the
            // realized delay rather than recomputing the duration.
            out.push((age, pool.swap_remove(i).item));
        } else {
            earliest = Some(earliest.map_or(enqueued_at, |e| e.min(enqueued_at)));
            i += 1;
        }
    }

    shuffle(out);
    earliest
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
    // range via `#[values(1.0, 5.0, 10.0)]`. At 512-byte packets these are ≈2k/10k/20k msg/s.

    fn entry<T>(item: T, age: Duration) -> Entry<T> {
        // Construct an entry as though it had been enqueued `age` ago.
        Entry::new(Instant::now() - age, item)
    }

    /// Engine params from explicit common fields plus a [`PoissonConfig`].
    fn params(min_delay: Duration, delay_range: Duration, poisson: PoissonConfig) -> PoissonParams {
        let cfg = MixerConfig {
            min_delay,
            delay_range,
            ..MixerConfig::default()
        };
        PoissonParams::new(&cfg, &poisson)
    }

    /// Default engine params (derived mean, default cap and watermarks).
    fn default_params() -> PoissonParams {
        PoissonParams::from_mixer(&MixerConfig::default())
    }

    /// Params with a cap ≈ 30 means, so truncation (`e^-30`) is negligible and shape assertions
    /// see the true exponential.
    fn untruncated_params(mean: Duration) -> PoissonParams {
        params(
            Duration::ZERO,
            mean * 30,
            PoissonConfig {
                target_mean_delay: mean,
                ..PoissonConfig::default()
            },
        )
    }

    /// One-shot `sweep` returning the released items, for the direct (non-simulation) tests.
    /// `delta` is irrelevant to the branches these exercise.
    fn sweep_once<T>(pool: &mut Vec<Entry<T>>, params: &PoissonParams) -> Vec<(Duration, T)> {
        let mut out = Vec::new();
        sweep(
            pool,
            params,
            Instant::now(),
            params.mean.max(Duration::from_millis(1)),
            &mut out,
        );
        out
    }

    #[test]
    fn passthrough_should_preserve_order() -> anyhow::Result<()> {
        let params = params(Duration::ZERO, Duration::ZERO, PoissonConfig::default());
        let mut pool: Vec<Entry<u32>> = (0..8).map(|i| entry(i, Duration::ZERO)).collect();

        let released: Vec<u32> = sweep_once(&mut pool, &params)
            .into_iter()
            .map(|(_, item)| item)
            .collect();

        assert!(pool.is_empty(), "passthrough must drain the whole pool");
        assert_eq!(
            released,
            (0..8).collect::<Vec<_>>(),
            "passthrough must preserve FIFO order"
        );
        Ok(())
    }

    #[test]
    fn deadline_should_force_release_regardless_of_rng() -> anyhow::Result<()> {
        let params = default_params();
        // Ages well beyond the cap — must always be released.
        let mut pool: Vec<Entry<u32>> = (0..16).map(|i| entry(i, params.cap + Duration::from_secs(1))).collect();

        let released = sweep_once(&mut pool, &params);

        assert!(pool.is_empty(), "all past-deadline items must be released");
        assert_eq!(released.len(), 16);
        Ok(())
    }

    #[test]
    fn small_buffer_should_enforce_minimum_dwell() -> anyhow::Result<()> {
        let params = default_params();
        let mean = params.mean;
        let occupancy = params.min_mix_occupancy as u32;
        assert!(occupancy >= 1);

        // At the threshold and all fresh (age below one mean): the minimum-dwell branch KEEPS
        // them — no coin, no early release.
        let mut pool: Vec<Entry<u32>> = (0..occupancy).map(|i| entry(i, mean / 4)).collect();
        assert!(
            sweep_once(&mut pool, &params).is_empty(),
            "small buffer must hold packets younger than one mean"
        );
        assert_eq!(pool.len(), params.min_mix_occupancy);

        // Once they have dwelt at least one mean, the same small buffer releases them.
        let mut pool: Vec<Entry<u32>> = (0..occupancy).map(|i| entry(i, mean + mean / 4)).collect();
        assert_eq!(
            sweep_once(&mut pool, &params).len(),
            params.min_mix_occupancy,
            "packets older than one mean are released"
        );
        Ok(())
    }

    #[rstest]
    fn delays_should_follow_exponential_distribution(#[values(1.0, 5.0, 10.0)] mb_per_s: f64) -> anyhow::Result<()> {
        // Across the realistic 1–10 MB/s load range, a relaxed cap makes truncation negligible
        // and the realized delays should exhibit the three exponential signatures.
        let params = untruncated_params(Duration::from_millis(10));
        let mean_ms = params.mean.as_secs_f64() * 1000.0;

        const N: usize = 6000;
        let sim = simulate_rate_driven(&params, N, inter_arrival_ms_for(mb_per_s), Duration::from_micros(25));
        let delays = steady_slice(&sim.delays_ms);

        let observed_mean = mean_of(delays);
        assert!(
            (observed_mean - mean_ms).abs() < 2.0,
            "[{mb_per_s} MB/s] mean {observed_mean:.2} ms should be ≈ {mean_ms:.2} ms"
        );

        // Signature 1: median = mean·ln2 ≈ 0.693·mean.
        let mut sorted = delays.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median = sorted[sorted.len() / 2];
        assert!(
            (0.55 * mean_ms..=0.85 * mean_ms).contains(&median),
            "[{mb_per_s} MB/s] median {median:.2} ms should be ≈ mean·ln2 = {:.2} ms",
            0.693 * mean_ms
        );

        // Signature 2: coefficient of variation ≈ 1.
        let cv = coefficient_of_variation(delays);
        assert!(
            (0.75..=1.25).contains(&cv),
            "[{mb_per_s} MB/s] CV {cv:.3} should be ≈ 1"
        );

        // Signature 3: real spread — an early head and a long tail.
        let min = sorted.first().copied().unwrap();
        let max = sorted.last().copied().unwrap();
        assert!(
            min < 3.0,
            "[{mb_per_s} MB/s] some packets should leave early (min {min:.2} ms)"
        );
        assert!(
            max > 2.0 * mean_ms,
            "[{mb_per_s} MB/s] some packets should leave late (max {max:.2} ms)"
        );
        Ok(())
    }

    /// Drive `n` packets through repeated sweeps advancing virtual time by `step`, returning
    /// their realized delays in milliseconds.
    ///
    /// Enqueue instants are dithered uniformly within one `step` so the packets have random
    /// phase relative to the sweep grid — as they would in reality, arriving at continuous
    /// wall-clock times. Without this, every delay lands on an exact multiple of `step`, whose
    /// alignment to distribution bin edges is a pure artifact of the discretization.
    fn simulate_delays_ms(params: &PoissonParams, n: usize, step: Duration) -> Vec<f64> {
        let t0 = Instant::now();
        let step_secs = step.as_secs_f64();
        let mut pool: Vec<Entry<u32>> = (0..n as u32)
            .map(|i| {
                let phase = Duration::from_secs_f64(crypto_random::random_float() * step_secs);
                Entry::new(t0 + phase, i)
            })
            .collect();

        let mut delays_ms = Vec::with_capacity(n);
        let mut out = Vec::new();
        let mut now = t0;
        let mut guard = 0;
        while !pool.is_empty() && guard < 1_000_000 {
            now += step;
            sweep(&mut pool, params, now, step, &mut out);
            for (d, _) in out.drain(..) {
                delays_ms.push(d.as_secs_f64() * 1000.0);
            }
            guard += 1;
        }
        assert!(
            pool.is_empty(),
            "simulate_delays_ms guard exhausted before the pool drained ({} packets left)",
            pool.len()
        );
        delays_ms
    }

    /// Bytes per packet used to translate a MB/s load into a message rate.
    const PACKET_BYTES: f64 = 512.0;

    /// Mean inter-arrival (ms) for a given load in MB/s at `PACKET_BYTES`-sized packets.
    fn inter_arrival_ms_for(mb_per_s: f64) -> f64 {
        1000.0 / (mb_per_s * 1_000_000.0 / PACKET_BYTES)
    }

    /// Outcome of a rate-driven simulation.
    struct RateSim {
        /// Realized delay (ms) of each packet, indexed by arrival order.
        delays_ms: Vec<f64>,
        /// Departure timestamps (ms from start), in departure order.
        departures_ms: Vec<f64>,
    }

    /// Drive `n` Poisson arrivals (exponential inter-arrivals of mean `inter_arrival_ms`)
    /// through the pool in virtual time, stepping by `step`. Arrival instants are continuous,
    /// so realized delays carry no grid-alignment artifact. This models a steady load rather
    /// than an instantaneous burst.
    fn simulate_rate_driven(params: &PoissonParams, n: usize, inter_arrival_ms: f64, step: Duration) -> RateSim {
        let mut arrivals = Vec::with_capacity(n);
        let mut t = 0.0;
        for _ in 0..n {
            let u = crypto_random::random_float();
            t += -inter_arrival_ms * (1.0 - u).ln();
            arrivals.push(t);
        }

        let t0 = Instant::now();
        let step_ms = step.as_secs_f64() * 1000.0;
        let mut pool: Vec<Entry<u32>> = Vec::new();
        let mut delays_ms = vec![0.0f64; n];
        let mut departures_ms = Vec::with_capacity(n);
        let mut out = Vec::new();
        let mut next = 0usize;
        let mut now_ms = 0.0f64;
        let mut guard = 0u64;
        while (next < n || !pool.is_empty()) && guard < 50_000_000 {
            now_ms += step_ms;
            while next < n && arrivals[next] <= now_ms {
                let enq = t0 + Duration::from_secs_f64(arrivals[next] / 1000.0);
                pool.push(Entry::new(enq, next as u32));
                next += 1;
            }
            let now = t0 + Duration::from_secs_f64(now_ms / 1000.0);
            sweep(&mut pool, params, now, step, &mut out);
            for (d, item) in out.drain(..) {
                delays_ms[item as usize] = d.as_secs_f64() * 1000.0;
                departures_ms.push(now_ms);
            }
            guard += 1;
        }
        assert!(
            next == n && pool.is_empty(),
            "simulate_rate_driven guard exhausted before all packets departed (arrived {next}/{n}, {} left in pool)",
            pool.len()
        );
        RateSim {
            delays_ms,
            departures_ms,
        }
    }

    /// Steady-state slice (middle 70%) of a series, discarding ramp-up/-down transients.
    fn steady_slice(v: &[f64]) -> &[f64] {
        let lo = v.len() * 15 / 100;
        let hi = v.len() * 85 / 100;
        &v[lo..hi]
    }

    fn mean_of(xs: &[f64]) -> f64 {
        xs.iter().sum::<f64>() / xs.len() as f64
    }

    fn coefficient_of_variation(xs: &[f64]) -> f64 {
        let m = mean_of(xs);
        let var = xs.iter().map(|x| (x - m).powi(2)).sum::<f64>() / xs.len() as f64;
        var.sqrt() / m
    }

    #[test]
    fn derived_mean_should_leave_two_percent_at_cap() -> anyhow::Result<()> {
        // mean = cap / ln(1/(1-0.98)) = 20 / ln(50) ≈ 5.11 ms should leave ~2% of packets to
        // be force-released at the 20 ms cap (i.e. 98% leave naturally within the cap).
        // Jitter off so the truncated mass lands exactly at the cap and is countable.
        let params = params(
            Duration::ZERO,
            Duration::from_millis(20),
            PoissonConfig {
                target_mean_delay: Duration::from_micros(5113),
                cap_jitter: Duration::ZERO,
                ..PoissonConfig::default()
            },
        );

        // Keep occupancy below `high_watermark` so the overload safety valve stays dormant
        // (a burst at `capacity` would legitimately trigger relief and flush early). `N` is large
        // enough that the ~2% force-release fraction has a tight standard error: at N = 8_000,
        // SE ≈ 0.0016, so the ±0.008 band below is ≈ ±5 sigma — negligible false-failure rate.
        const N: usize = 8_000;
        assert!(N < params.high_watermark);
        let cap_ms = params.cap.as_secs_f64() * 1000.0;
        let delays_ms = simulate_delays_ms(&params, N, Duration::from_micros(250));
        assert_eq!(delays_ms.len(), N);

        let at_cap = delays_ms.iter().filter(|d| **d >= cap_ms).count();
        let frac = at_cap as f64 / N as f64;
        assert!(
            (0.012..=0.028).contains(&frac),
            "expected ~2% force-released at the cap, got {:.2}%",
            frac * 100.0
        );

        let observed_mean = delays_ms.iter().sum::<f64>() / N as f64;
        assert!(
            (4.6..=5.6).contains(&observed_mean),
            "observed mean {observed_mean:.2} ms should be near 5.11 ms"
        );
        Ok(())
    }

    #[test]
    fn saturation_should_preserve_minimum_delay() -> anyhow::Result<()> {
        // A burst at full capacity trips the overload valve. With the saturation floor the
        // effective mean bottoms out at `saturation_min_mean` rather than zero, so packets
        // still incur a non-trivial mixing delay instead of passing straight through.
        let params = default_params();
        let n = params.capacity;
        let floor_ms = params.saturation_min_mean.as_secs_f64() * 1000.0;

        let delays_ms = simulate_delays_ms(&params, n, Duration::from_micros(250));
        assert_eq!(delays_ms.len(), n, "every packet must eventually be released");

        let observed_mean = delays_ms.iter().sum::<f64>() / n as f64;
        // Without the floor the valve would flush the whole burst in the first sweep
        // (mean ≈ one step ≈ 0.25 ms). The floor keeps the mean on the order of milliseconds.
        assert!(
            observed_mean > floor_ms * 0.5,
            "saturation must preserve a minimum delay (observed mean {observed_mean:.2} ms, floor {floor_ms:.2} ms)"
        );
        Ok(())
    }

    #[rstest]
    fn holding_time_should_be_memoryless(#[values(1.0, 5.0, 10.0)] mb_per_s: f64) -> anyhow::Result<()> {
        // Memorylessness ⇒ the survival function decays at a constant rate:
        // P(X > 2m) / P(X > m) = P(X > m) = e^-1 ≈ 0.368, independent of the offset.
        let params = untruncated_params(Duration::from_millis(10));
        let mean_ms = params.mean.as_secs_f64() * 1000.0;

        const N: usize = 8_000;
        let sim = simulate_rate_driven(&params, N, inter_arrival_ms_for(mb_per_s), Duration::from_micros(50));
        let delays = steady_slice(&sim.delays_ms);

        let past_1m = delays.iter().filter(|d| **d > mean_ms).count() as f64;
        let past_2m = delays.iter().filter(|d| **d > 2.0 * mean_ms).count() as f64;
        assert!(past_1m > 0.0);
        let ratio = past_2m / past_1m;
        assert!(
            (0.28..=0.46).contains(&ratio),
            "[{mb_per_s} MB/s] conditional survival ratio {ratio:.3} should be ≈ e^-1 = 0.368 (memoryless)"
        );
        Ok(())
    }

    #[rstest]
    fn delay_distribution_should_be_independent_of_tick_cadence(
        #[values(1.0, 5.0, 10.0)] mb_per_s: f64,
    ) -> anyhow::Result<()> {
        // The continuous exponential clock `1 - e^(-δ/mean)` makes the distribution independent
        // of the sweep cadence: a fine and a coarse step must yield the same mean at any load.
        let params = untruncated_params(Duration::from_millis(10));
        const N: usize = 8_000;
        let inter = inter_arrival_ms_for(mb_per_s);

        let fine = simulate_rate_driven(&params, N, inter, Duration::from_micros(50));
        let coarse = simulate_rate_driven(&params, N, inter, Duration::from_micros(500));

        let mean_fine = mean_of(steady_slice(&fine.delays_ms));
        let mean_coarse = mean_of(steady_slice(&coarse.delays_ms));
        let rel_diff = (mean_fine - mean_coarse).abs() / mean_fine;
        assert!(
            rel_diff < 0.2,
            "[{mb_per_s} MB/s] mean should be cadence-independent: fine {mean_fine:.2} ms vs coarse {mean_coarse:.2} \
             ms"
        );
        Ok(())
    }

    #[rstest]
    fn delays_should_pass_chi_squared_fit_to_exponential(
        #[values(1.0, 5.0, 10.0)] mb_per_s: f64,
    ) -> anyhow::Result<()> {
        // Bin the realized delays by the exponential CDF (probability-integral transform:
        // u = 1 - e^(-d/mean) is Uniform[0,1] iff d is Exp(mean)) into K equiprobable bins and
        // run a Pearson chi-squared test. df = K-1 = 9; the threshold 40 corresponds to
        // p ≈ 1e-5, so a true exponential (expected χ² ≈ 9) passes with wide margin while a
        // wrong distribution (uniform, deterministic, …) produces χ² in the hundreds.
        let params = untruncated_params(Duration::from_millis(10));
        let mean_ms = params.mean.as_secs_f64() * 1000.0;

        const N: usize = 8_000;
        const K: usize = 10;
        let sim = simulate_rate_driven(&params, N, inter_arrival_ms_for(mb_per_s), Duration::from_micros(25));
        let delays = steady_slice(&sim.delays_ms);

        let mut counts = [0usize; K];
        for &d in delays {
            let u = 1.0 - (-d / mean_ms).exp();
            let bin = ((u * K as f64) as usize).min(K - 1);
            counts[bin] += 1;
        }

        let expected = delays.len() as f64 / K as f64;
        let chi_sq: f64 = counts
            .iter()
            .map(|&o| {
                let diff = o as f64 - expected;
                diff * diff / expected
            })
            .sum();

        assert!(
            chi_sq < 40.0,
            "[{mb_per_s} MB/s] χ² = {chi_sq:.2} (df={}) exceeds the goodness-of-fit threshold; delays are not \
             exponential (bins: {counts:?})",
            K - 1
        );
        Ok(())
    }

    #[rstest]
    fn poisson_arrivals_should_yield_poisson_departures(#[values(1.0, 5.0, 10.0)] mb_per_s: f64) -> anyhow::Result<()> {
        // Displacement theorem: Poisson arrivals + i.i.d. exponential delays ⇒ the departure
        // process is Poisson at the same rate. Check the departures for (a) rate conservation
        // and (b) the Poisson signature CV ≈ 1 on inter-departure gaps, across the load range.
        let params = untruncated_params(Duration::from_millis(10));
        let inter_arrival_ms = inter_arrival_ms_for(mb_per_s);

        const N: usize = 6_000;
        let sim = simulate_rate_driven(&params, N, inter_arrival_ms, Duration::from_micros(25));
        assert_eq!(sim.departures_ms.len(), N, "every packet must depart");

        let steady = steady_slice(&sim.departures_ms);
        let inter: Vec<f64> = steady.windows(2).map(|w| w[1] - w[0]).collect();
        let mean_id = mean_of(&inter);

        // (a) Rate conservation: mean inter-departure ≈ mean inter-arrival.
        assert!(
            (mean_id - inter_arrival_ms).abs() / inter_arrival_ms < 0.25,
            "[{mb_per_s} MB/s] output rate should match input: mean inter-departure {mean_id:.4} ms vs inter-arrival \
             {inter_arrival_ms:.4} ms"
        );

        // (b) Poisson signature: inter-departure coefficient of variation ≈ 1.
        let cv = coefficient_of_variation(&inter);
        assert!(
            (0.6..=1.6).contains(&cv),
            "[{mb_per_s} MB/s] inter-departure CV {cv:.3} should be ≈ 1 for a Poisson output process"
        );
        Ok(())
    }

    #[test]
    fn min_delay_floor_should_keep_ineligible_items() -> anyhow::Result<()> {
        let params = params(
            Duration::from_millis(50),
            Duration::from_millis(20),
            PoissonConfig::default(),
        );
        // Age below the floor: must be kept even though the pool is tiny.
        let mut pool: Vec<Entry<u32>> = vec![entry(1u32, Duration::from_millis(5))];

        let released = sweep_once(&mut pool, &params);

        assert_eq!(pool.len(), 1, "item under the min-delay floor must be kept");
        assert!(released.is_empty());
        Ok(())
    }

    #[rstest]
    #[case(Duration::ZERO, Duration::from_millis(10), 0.0)]
    #[case(Duration::from_millis(10), Duration::from_millis(10), 0.6321)] // 1 - e^-1 after one mean
    #[case(Duration::from_millis(5), Duration::ZERO, 1.0)] // degenerate mean releases unconditionally
    fn release_probability_should_follow_the_exponential_clock(
        #[case] delta: Duration,
        #[case] mean: Duration,
        #[case] expected: f64,
    ) {
        assert!((release_probability(delta, mean) - expected).abs() < 1e-3);
    }

    #[test]
    fn next_wake_should_be_the_idle_heartbeat_when_the_pool_is_empty() {
        assert_eq!(next_wake(None, 0, &default_params(), Instant::now()), IDLE_HEARTBEAT);
    }

    #[test]
    fn next_wake_should_be_floored_at_min_wake_for_an_overdue_packet() {
        // A packet already past its jitter-window opening drives `deadline_wake` to zero, so the
        // result is the `MIN_WAKE` floor rather than a busy-spin at zero.
        let params = default_params();
        let now = Instant::now();
        let overdue = now - params.cap - Duration::from_secs(1);
        assert_eq!(next_wake(Some(overdue), 1000, &params, now), MIN_WAKE);
    }

    #[test]
    fn adaptive_interval_should_shorten_with_occupancy() {
        let params = default_params();
        // One item: the ceiling (one mean); many items: the floor.
        assert!((params.adaptive_interval(1).as_secs_f64() - params.mean.as_secs_f64()).abs() < 1e-6);
        assert_eq!(params.adaptive_interval(100_000), params.tick_floor);
        assert!(params.adaptive_interval(10) >= params.adaptive_interval(100));
    }

    #[test]
    fn mean_for_should_shrink_from_the_base_toward_the_floor_above_the_watermark() {
        let params = default_params();
        assert_eq!(params.mean_for(params.high_watermark), params.mean);
        assert!(params.mean_for(params.high_watermark + 1) < params.mean);
        assert_eq!(params.mean_for(params.capacity), params.saturation_min_mean);
    }

    #[test]
    fn derived_mean_should_use_the_delay_range_not_the_cap() {
        // min_delay = 10 ms, delay_range = 20 ms ⇒ mean = 20 / ln(20) ≈ 6.68 ms (the eligible
        // window is the range, not the 30 ms cap).
        let params = params(
            Duration::from_millis(10),
            Duration::from_millis(20),
            PoissonConfig::default(),
        );
        assert!((params.mean.as_secs_f64() * 1000.0 - 6.68).abs() < 0.1);
    }

    #[test]
    fn explicit_target_mean_should_override_derivation() {
        let params = params(
            Duration::ZERO,
            Duration::from_millis(20),
            PoissonConfig {
                target_mean_delay: Duration::from_millis(10),
                ..PoissonConfig::default()
            },
        );
        assert_eq!(params.mean, Duration::from_millis(10));
    }
}
