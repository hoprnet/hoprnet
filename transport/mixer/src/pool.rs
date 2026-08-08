//! Pure buffer-and-sweep logic for the exponential (Poisson) release engine.
//!
//! This module is deliberately free of any async, threading or channel concerns so
//! that the release decision can be unit-tested in isolation. The engine in
//! [`crate::poisson`] owns a `Vec<Entry<T>>` and calls [`sweep`] on every wake.

use std::time::{Duration, Instant};

use hopr_types::crypto_random;

use crate::config::MixerConfig;

/// Idle heartbeat used when the pool is empty, so an engine periodically notices a
/// dropped receiver and can shut down instead of parking forever. Shared by both engines
/// (dedicated-thread and shared-pool) as the initial timer duration and the empty-pool wake.
pub(crate) const IDLE_HEARTBEAT: Duration = Duration::from_millis(200);
/// Lower bound on any computed wake, preventing a busy-spin near a packet deadline.
const MIN_WAKE: Duration = Duration::from_micros(100);

/// Compute the next wake from the current occupancy and the earliest still-buffered enqueue
/// instant (as returned by [`sweep`], so no extra O(N) scan): the adaptive interval, capped by
/// the soonest moment any packet's jitter window opens, floored at [`MIN_WAKE`]; an idle
/// heartbeat when the pool is empty. Engine-agnostic, so both engines share one wake policy.
pub(crate) fn next_wake(
    earliest_enqueued: Option<Instant>,
    occupancy: usize,
    cfg: &MixerConfig,
    now: Instant,
) -> Duration {
    match earliest_enqueued {
        None => IDLE_HEARTBEAT,
        Some(enqueued_at) => {
            let interval = cfg.adaptive_interval(occupancy);
            let window_opens = cfg.cap().saturating_sub(cfg.cap_jitter);
            let deadline_wake = (enqueued_at + window_opens).saturating_duration_since(now);
            interval.min(deadline_wake).max(MIN_WAKE)
        }
    }
}

/// A single buffered item awaiting release.
pub(crate) struct Entry<T> {
    /// When the item entered the mixer.
    pub enqueued_at: Instant,
    /// The buffered payload.
    pub item: T,
}

impl<T> Entry<T> {
    pub(crate) fn new(enqueued_at: Instant, item: T) -> Self {
        Self { enqueued_at, item }
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
/// 2. `age >= cap`, or `age + U[0, cap_jitter] >= cap` within the jitter window — hard-cap force-release
///    (jitter-smeared).
/// 3. `occupancy <= min_mix_occupancy` — small buffer: deterministic minimum dwell, released once `age >= mean` (no
///    coin), so the few packets present overlap and mix instead of escaping through the exponential's fast tail.
/// 4. `Bernoulli(1 - e^(-delta/mean))` — the memoryless exponential clock.
///
/// When no delay is configured (`is_passthrough`) the pool is drained in enqueue order without
/// mixing, preserving FIFO semantics.
///
/// Performance: the release probability is memoryless and depends only on `delta` (shared by
/// the whole pool this tick), so `1 - e^(-delta/mean)` is computed **once** rather than per
/// packet; and the jitter draw is taken only for packets actually inside the `[cap-jitter, cap)`
/// window.
pub(crate) fn sweep<T>(
    pool: &mut Vec<Entry<T>>,
    cfg: &MixerConfig,
    now: Instant,
    delta: Duration,
    out: &mut Vec<(Duration, T)>,
) -> Option<Instant> {
    out.clear();
    if pool.is_empty() {
        return None;
    }

    if cfg.is_passthrough() {
        for e in pool.drain(..) {
            out.push((now.saturating_duration_since(e.enqueued_at), e.item));
        }
        return None;
    }

    let occupancy = pool.len();
    let cap = cfg.cap();
    let jitter = cfg.cap_jitter;
    let min_delay = cfg.min_delay;
    let min_mix_occupancy = cfg.min_mix_occupancy;
    let mean = cfg.mean_for(occupancy);

    // Computed once per sweep (memoryless, shared `delta`) rather than per packet.
    let release_probability = cfg.release_probability(delta, mean);
    let jitter_window_start = cap.saturating_sub(jitter);

    let mut earliest: Option<Instant> = None;
    let mut i = 0;
    while i < pool.len() {
        let enqueued_at = pool[i].enqueued_at;
        let age = now.saturating_duration_since(enqueued_at);

        let release = if age < min_delay {
            false
        } else if age >= cap || (age >= jitter_window_start && reached_cap_in_window(age, cap, jitter)) {
            // Hard-cap force-release: past the cap, or inside the jitter window and sampled to
            // release. `||` short-circuits so the jitter draw is skipped once `age >= cap`.
            true
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
                release_probability
            } else {
                cfg.release_probability(eligible_for, mean)
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

/// Whether an item already known to be inside the `[cap-jitter, cap)` window is force-released,
/// drawing the jitter sample only here (never for packets far from the cap). Release iff
/// `age + U[0, jitter] >= cap`.
fn reached_cap_in_window(age: Duration, cap: Duration, jitter: Duration) -> bool {
    if jitter.is_zero() {
        return false;
    }
    let sample = Duration::from_secs_f64(crypto_random::random_float_in_range(0.0..jitter.as_secs_f64()));
    age.saturating_add(sample) >= cap
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
        Entry {
            enqueued_at: Instant::now() - age,
            item,
        }
    }

    /// One-shot `sweep` wrapper returning the released items as a `Vec`, for the direct
    /// (non-simulation) unit tests. `delta` is irrelevant to the branches these tests exercise.
    fn sweep_once<T>(pool: &mut Vec<Entry<T>>, cfg: &MixerConfig, now: Instant) -> Vec<(Duration, T)> {
        let mut out = Vec::new();
        sweep(pool, cfg, now, cfg.mean().max(Duration::from_millis(1)), &mut out);
        out
    }

    #[test]
    fn passthrough_should_preserve_order() -> anyhow::Result<()> {
        let cfg = MixerConfig {
            min_delay: Duration::ZERO,
            delay_range: Duration::ZERO,
            ..MixerConfig::default()
        };
        let mut pool: Vec<Entry<u32>> = (0..8).map(|i| entry(i, Duration::ZERO)).collect();

        let released: Vec<u32> = sweep_once(&mut pool, &cfg, Instant::now())
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
        let cfg = MixerConfig::default();
        // Ages well beyond the cap — must always be released.
        let mut pool: Vec<Entry<u32>> = (0..16).map(|i| entry(i, cfg.cap() + Duration::from_secs(1))).collect();

        let released = sweep_once(&mut pool, &cfg, Instant::now());

        assert!(pool.is_empty(), "all past-deadline items must be released");
        assert_eq!(released.len(), 16);
        Ok(())
    }

    #[test]
    fn small_buffer_should_enforce_minimum_dwell() -> anyhow::Result<()> {
        let cfg = MixerConfig::default();
        let mean = cfg.mean();
        assert!(cfg.min_mix_occupancy >= 1);

        // Occupancy at the threshold, all fresh (age well below one mean): the deterministic
        // minimum-dwell branch must KEEP them — no coin, no early release.
        let young_age = mean / 4;
        let mut pool: Vec<Entry<u32>> = (0..cfg.min_mix_occupancy as u32).map(|i| entry(i, young_age)).collect();
        let released = sweep_once(&mut pool, &cfg, Instant::now());
        assert!(
            released.is_empty(),
            "small buffer must hold packets younger than one mean"
        );
        assert_eq!(pool.len(), cfg.min_mix_occupancy);

        // Once they have dwelt at least one mean, the same small buffer releases them.
        let old_age = mean + mean / 4;
        let mut pool: Vec<Entry<u32>> = (0..cfg.min_mix_occupancy as u32).map(|i| entry(i, old_age)).collect();
        let released = sweep_once(&mut pool, &cfg, Instant::now());
        assert_eq!(
            released.len(),
            cfg.min_mix_occupancy,
            "packets older than one mean are released"
        );
        Ok(())
    }

    #[rstest]
    fn delays_should_follow_exponential_distribution(#[values(1.0, 5.0, 10.0)] mb_per_s: f64) -> anyhow::Result<()> {
        // Across the realistic 1–10 MB/s load range, a relaxed cap makes truncation negligible
        // and the realized delays should exhibit the three exponential signatures.
        let cfg = untruncated_cfg(Duration::from_millis(10));
        let mean_ms = cfg.mean().as_secs_f64() * 1000.0;

        const N: usize = 6000;
        let sim = simulate_rate_driven(&cfg, N, inter_arrival_ms_for(mb_per_s), Duration::from_micros(25));
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
    fn simulate_delays_ms(cfg: &MixerConfig, n: usize, step: Duration) -> Vec<f64> {
        let t0 = Instant::now();
        let step_secs = step.as_secs_f64();
        let mut pool: Vec<Entry<u32>> = (0..n as u32)
            .map(|i| {
                let phase = Duration::from_secs_f64(crypto_random::random_float() * step_secs);
                Entry {
                    enqueued_at: t0 + phase,
                    item: i,
                }
            })
            .collect();

        let mut delays_ms = Vec::with_capacity(n);
        let mut out = Vec::new();
        let mut now = t0;
        let mut guard = 0;
        while !pool.is_empty() && guard < 1_000_000 {
            now += step;
            sweep(&mut pool, cfg, now, step, &mut out);
            for (d, _) in out.drain(..) {
                delays_ms.push(d.as_secs_f64() * 1000.0);
            }
            guard += 1;
        }
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
    fn simulate_rate_driven(cfg: &MixerConfig, n: usize, inter_arrival_ms: f64, step: Duration) -> RateSim {
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
                pool.push(Entry {
                    enqueued_at: enq,
                    item: next as u32,
                });
                next += 1;
            }
            let now = t0 + Duration::from_secs_f64(now_ms / 1000.0);
            sweep(&mut pool, cfg, now, step, &mut out);
            for (d, item) in out.drain(..) {
                delays_ms[item as usize] = d.as_secs_f64() * 1000.0;
                departures_ms.push(now_ms);
            }
            guard += 1;
        }
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
        let cfg = MixerConfig {
            target_mean_delay: Duration::from_micros(5113),
            min_delay: Duration::ZERO,
            delay_range: Duration::from_millis(20),
            cap_jitter: Duration::ZERO,
            ..MixerConfig::default()
        };

        // Keep occupancy below `high_watermark` so the overload safety valve stays dormant
        // (a burst at `capacity` would legitimately trigger relief and flush early).
        const N: usize = 5_000;
        assert!(N < cfg.high_watermark);
        let cap_ms = cfg.cap().as_secs_f64() * 1000.0;
        let delays_ms = simulate_delays_ms(&cfg, N, Duration::from_micros(250));
        assert_eq!(delays_ms.len(), N);

        let at_cap = delays_ms.iter().filter(|d| **d >= cap_ms).count();
        let frac = at_cap as f64 / N as f64;
        assert!(
            (0.013..=0.027).contains(&frac),
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
        let cfg = MixerConfig::default();
        let n = cfg.capacity;
        let floor_ms = cfg.saturation_min_mean.as_secs_f64() * 1000.0;

        let delays_ms = simulate_delays_ms(&cfg, n, Duration::from_micros(250));
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

    /// Config with a relaxed cap so the exponential tail is essentially untruncated,
    /// letting distribution-shape assertions see the true exponential.
    fn untruncated_cfg(mean: Duration) -> MixerConfig {
        MixerConfig {
            target_mean_delay: mean,
            min_delay: Duration::ZERO,
            delay_range: mean * 30, // cap ≈ 30 means → truncation e^-30 ≈ 0
            ..MixerConfig::default()
        }
    }

    #[rstest]
    fn holding_time_should_be_memoryless(#[values(1.0, 5.0, 10.0)] mb_per_s: f64) -> anyhow::Result<()> {
        // Memorylessness ⇒ the survival function decays at a constant rate:
        // P(X > 2m) / P(X > m) = P(X > m) = e^-1 ≈ 0.368, independent of the offset.
        let cfg = untruncated_cfg(Duration::from_millis(10));
        let mean_ms = cfg.mean().as_secs_f64() * 1000.0;

        const N: usize = 8_000;
        let sim = simulate_rate_driven(&cfg, N, inter_arrival_ms_for(mb_per_s), Duration::from_micros(50));
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
        let cfg = untruncated_cfg(Duration::from_millis(10));
        const N: usize = 8_000;
        let inter = inter_arrival_ms_for(mb_per_s);

        let fine = simulate_rate_driven(&cfg, N, inter, Duration::from_micros(50));
        let coarse = simulate_rate_driven(&cfg, N, inter, Duration::from_micros(500));

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
        let cfg = untruncated_cfg(Duration::from_millis(10));
        let mean_ms = cfg.mean().as_secs_f64() * 1000.0;

        const N: usize = 8_000;
        const K: usize = 10;
        let sim = simulate_rate_driven(&cfg, N, inter_arrival_ms_for(mb_per_s), Duration::from_micros(25));
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
        let cfg = untruncated_cfg(Duration::from_millis(10));
        let inter_arrival_ms = inter_arrival_ms_for(mb_per_s);

        const N: usize = 6_000;
        let sim = simulate_rate_driven(&cfg, N, inter_arrival_ms, Duration::from_micros(25));
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
        let cfg = MixerConfig {
            min_delay: Duration::from_millis(50),
            delay_range: Duration::from_millis(20),
            ..MixerConfig::default()
        };
        // Age below the floor: must be kept even though the pool is tiny.
        let mut pool: Vec<Entry<u32>> = vec![entry(1u32, Duration::from_millis(5))];

        let released = sweep_once(&mut pool, &cfg, Instant::now());

        assert_eq!(pool.len(), 1, "item under the min-delay floor must be kept");
        assert!(released.is_empty());
        Ok(())
    }
}
