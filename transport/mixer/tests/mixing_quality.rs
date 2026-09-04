//! End-to-end mixing-quality checks for the shared-pool virtual-clock timing-wheel mixer.
//!
//! `src/pool.rs` unit-tests the release mechanism directly against pure functions; this suite
//! drives the real `poisson_channel` API (real tokio tasks, real wall-clock sends) so a
//! wiring bug in `poisson.rs` itself — the `enqueue`/`sweep`/`next_wake` call sites, the
//! lock discipline, the wake scheduling — has somewhere to be caught that the pure-function tests
//! cannot see.

#![cfg(feature = "poisson")]

use std::time::{Duration, Instant};

use futures::StreamExt;
use futures_timer::Delay;
use hopr_transport_mixer::{MixerConfig, poisson_channel};

const COUNT: usize = 5_000;
/// ~10k msg/s ingress — the top of the target regime.
const SEND_SPACING: Duration = Duration::from_micros(100);

fn percentile(sorted_ms: &[f64], p: f64) -> f64 {
    if sorted_ms.is_empty() {
        return 0.0;
    }
    let idx = ((p * (sorted_ms.len() - 1) as f64).round() as usize).min(sorted_ms.len() - 1);
    sorted_ms[idx]
}

fn mean(xs: &[f64]) -> f64 {
    xs.iter().sum::<f64>() / xs.len() as f64
}

fn poisson_cfg(max_delay: Duration, miss_probability: f64, target_occupancy: usize) -> MixerConfig {
    MixerConfig::new_poisson_constant_privacy(max_delay, miss_probability, target_occupancy)
}

/// Realized-delay statistics measured end-to-end through the channel, at a fixed send spacing.
struct Stats {
    delays_ms: Vec<f64>,
    out_of_order_frac: f64,
}

fn run_scenario(cfg: MixerConfig, count: usize, spacing: Duration) -> Stats {
    let (mut delays_ms, out_of_order) = futures::executor::block_on(async move {
        let (tx, mut rx) = poisson_channel::<(u32, Instant)>(cfg);

        let sender = async move {
            for seq in 0..count as u32 {
                tx.send((seq, Instant::now())).expect("send must succeed");
                Delay::new(spacing).await;
            }
            // Dropping `tx` here closes the ingress so the receiver eventually sees `None`.
        };

        let receiver = async move {
            let mut delays_ms = Vec::with_capacity(count);
            let mut max_seq_seen: i64 = -1;
            let mut out_of_order = 0usize;
            while let Some((seq, sent_at)) = rx.next().await {
                delays_ms.push(sent_at.elapsed().as_secs_f64() * 1000.0);
                if (seq as i64) < max_seq_seen {
                    out_of_order += 1;
                }
                max_seq_seen = max_seq_seen.max(seq as i64);
            }
            (delays_ms, out_of_order)
        };

        let ((), result) = futures::join!(sender, receiver);
        result
    });

    let n = delays_ms.len();
    assert_eq!(n, count, "every enqueued packet must be delivered before close");
    delays_ms.sort_by(|a, b| a.partial_cmp(b).unwrap());

    Stats {
        delays_ms,
        out_of_order_frac: out_of_order as f64 / n as f64,
    }
}

/// The hard bound holds end-to-end, and mixing (reordering) actually occurs, in bounded-latency
/// mode (`target_occupancy = 0`, the shipped default).
#[test]
fn bounded_latency_channel_should_respect_the_hard_bound_and_mix() {
    let max_delay = Duration::from_millis(20);
    let cfg = poisson_cfg(max_delay, 0.01, 0);
    let stats = run_scenario(cfg, COUNT, SEND_SPACING);

    assert!(
        stats.out_of_order_frac > 0.1,
        "expected substantial reordering, got {:.1}%",
        100.0 * stats.out_of_order_frac
    );

    let max_delay_ms = max_delay.as_secs_f64() * 1000.0;
    let max = stats.delays_ms.last().copied().unwrap_or(0.0);
    // Generous scheduling slack: this is real wall-clock tokio scheduling, not the pure-function
    // simulation in `pool.rs`, which asserts the bound far more tightly.
    assert!(
        max < max_delay_ms + 100.0,
        "max realized delay {max:.2} ms should stay within {max_delay_ms} ms plus scheduling slack"
    );

    let observed_mean = mean(&stats.delays_ms);
    assert!(
        (2.0..=20.0).contains(&observed_mean),
        "bounded-latency mean {observed_mean:.2} ms outside the expected [2, 20] ms band"
    );
}

/// Constant-privacy mode (`target_occupancy > 0`) hits the design target end-to-end: ~10 ms mean
/// at 1000 pkt/s with `target_occupancy = 14`, `max_delay = 200 ms`.
#[test]
fn constant_privacy_channel_should_hit_the_design_target_at_1000_pps() {
    let cfg = poisson_cfg(Duration::from_millis(200), 0.01, 14);
    // 1000 pkt/s => 1ms spacing.
    let stats = run_scenario(cfg, 4_000, Duration::from_millis(1));
    let observed_mean = mean(&stats.delays_ms);
    assert!(
        (5.0..=17.0).contains(&observed_mean),
        "constant-privacy mean at 1000 pkt/s should be near 10ms (target_occupancy=14, max_delay=200ms), got \
         {observed_mean:.2}ms"
    );
}

/// A far looser bound truncates less than a tight one, at the same mean-anchoring approach used
/// throughout the design — a sanity check that `max_delay`/`miss_probability` actually drive the
/// observed tail rather than being ignored by the wiring.
#[test]
fn a_tighter_bound_should_truncate_the_tail_more_than_a_looser_one() {
    let tight = run_scenario(poisson_cfg(Duration::from_millis(20), 0.01, 0), COUNT, SEND_SPACING);
    let loose = run_scenario(poisson_cfg(Duration::from_millis(100), 0.01, 0), COUNT, SEND_SPACING);

    let tight_p99 = percentile(&tight.delays_ms, 0.99);
    let loose_p99 = percentile(&loose.delays_ms, 0.99);
    assert!(
        tight_p99 < loose_p99,
        "tight-bound p99 {tight_p99:.2} ms should be below loose-bound p99 {loose_p99:.2} ms"
    );

    // Nothing is delivered meaningfully past either bound.
    let tight_max = tight.delays_ms.last().copied().unwrap_or(0.0);
    assert!(
        tight_max <= 20.0 + 100.0,
        "p100 {tight_max:.2} ms should not exceed the 20ms bound by more than scheduling slack"
    );
}

/// Passthrough (`max_delay = Duration::ZERO`) preserves FIFO order end-to-end, the one case where
/// reordering must NOT occur.
#[test]
fn passthrough_channel_should_preserve_order() {
    const ITERATIONS: usize = 40;
    let cfg = poisson_cfg(Duration::ZERO, 0.01, 0);

    futures::executor::block_on(async move {
        let (tx, rx) = poisson_channel(cfg);
        let input: Vec<u32> = (0..ITERATIONS as u32).collect();
        for i in &input {
            tx.send(*i).expect("send must succeed");
        }
        drop(tx);
        let output: Vec<u32> = rx.take(ITERATIONS).collect().await;
        assert_eq!(input, output, "pass-through must preserve FIFO order");
    });
}
