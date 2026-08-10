//! Mixing-quality measurement for the exponential (Poisson) release engine.
//!
//! Feeds a steady stream of packets through `poisson_channel`, records each packet's
//! realized end-to-end delay and whether the output order was mixed, and prints the
//! resulting distribution. Unlike the criterion throughput bench (which only measures
//! wall-time), this exercises the property the engine actually exists for.
//!
//! Run with:
//! ```text
//! cargo run --release -p hopr-transport-mixer --example mixing_quality
//! ```

use std::time::{Duration, Instant};

use futures::StreamExt;
use futures_timer::Delay;
use hopr_transport_mixer::{MixerConfig, MixerType, PoissonConfig, PoissonDelay, poisson_channel};

const COUNT: usize = 20_000;
/// ~10k msg/s ingress — the top of the target regime.
const SEND_SPACING: Duration = Duration::from_micros(100);

fn percentile(sorted_ms: &[f64], p: f64) -> f64 {
    if sorted_ms.is_empty() {
        return 0.0;
    }
    let idx = ((p * (sorted_ms.len() - 1) as f64).round() as usize).min(sorted_ms.len() - 1);
    sorted_ms[idx]
}

/// A dedicated-thread Poisson config with the given delay anchor, percentile, and jitter.
fn poisson_cfg(delay: PoissonDelay, cap_percentile: f64, cap_jitter: Duration) -> MixerConfig {
    MixerConfig {
        mixer_type: MixerType::Poisson(PoissonConfig {
            delay,
            cap_percentile,
            cap_jitter,
            ..PoissonConfig::default()
        }),
        ..MixerConfig::default()
    }
}

fn run_scenario(label: &str, cfg: MixerConfig) {
    let (cap_ms, mean_ms) = match cfg.mixer_type {
        MixerType::Poisson(pc) => {
            let (cap, mean) = pc.delay.resolve(pc.cap_percentile);
            (cap.as_secs_f64() * 1000.0, mean.as_secs_f64() * 1000.0)
        }
        #[allow(unreachable_patterns)]
        _ => (0.0, 0.0),
    };

    let (delays_ms, out_of_order) = futures::executor::block_on(async move {
        let (tx, mut rx) = poisson_channel::<(u32, Instant)>(cfg);

        let sender = async move {
            for seq in 0..COUNT as u32 {
                tx.send((seq, Instant::now())).expect("send must succeed");
                Delay::new(SEND_SPACING).await;
            }
            // Dropping `tx` here closes the ingress so the receiver eventually sees `None`.
        };

        let receiver = async move {
            let mut delays_ms = Vec::with_capacity(COUNT);
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
    let mut sorted = delays_ms.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let sum: f64 = delays_ms.iter().sum();
    let observed_mean = sum / n as f64;
    // Observed end-to-end delay at or above the cap. This is a measured delay, not a release-path
    // label: receiver scheduling can push a probabilistic release to `>= cap` too, so it only
    // upper-bounds the true force-release count rather than equalling it.
    let at_or_over_cap = delays_ms.iter().filter(|d| **d >= cap_ms - 1e-6).count();

    println!("── {label} ─────────────────────────────────────────────");
    println!("  config:      mean(target)={mean_ms:.2} ms   cap={cap_ms:.0} ms   n={n}");
    println!("  observed:    mean={observed_mean:.2} ms");
    println!(
        "  percentiles: p50={:.2}  p90={:.2}  p95={:.2}  p99={:.2}  max={:.2} ms",
        percentile(&sorted, 0.50),
        percentile(&sorted, 0.90),
        percentile(&sorted, 0.95),
        percentile(&sorted, 0.99),
        percentile(&sorted, 1.0),
    );
    println!(
        "  at cap:      {at_or_over_cap} packets ({:.2}%) with observed delay >= cap",
        100.0 * at_or_over_cap as f64 / n as f64
    );
    println!(
        "  mixing:      {out_of_order} of {n} arrived out of send order ({:.1}%)",
        100.0 * out_of_order as f64 / n as f64
    );
    println!();
}

fn main() {
    println!(
        "Mixing-quality measurement: {COUNT} packets at ~{:.0}k msg/s, mean = 10 ms\n",
        1.0 / SEND_SPACING.as_secs_f64() / 1000.0
    );

    // Anchor the mean at 10 ms and vary the cap via the percentile.
    let mean = Duration::from_millis(10);
    let default_jitter = Duration::from_millis(2);

    // mean 10 ms, cap 20 ms ⇒ 1 - e^(-20/10) ≈ 86.5th percentile.
    run_scenario(
        "mean 10 ms, cap ~20 ms",
        poisson_cfg(PoissonDelay::Mean(mean), 0.865, default_jitter),
    );

    // mean 10 ms, cap 100 ms ⇒ ~99.995th percentile (relaxed cap, negligible truncation).
    run_scenario(
        "mean 10 ms, cap ~100 ms (relaxed)",
        poisson_cfg(PoissonDelay::Mean(mean), 0.99995, default_jitter),
    );

    // Cap 20 ms at the 98th percentile ⇒ mean ≈ 5.11 ms, ~2% force-released. Jitter off so the
    // truncated mass lands exactly at the cap and is countable.
    run_scenario(
        "cap 20 ms @ 98%: mean ~5.11 ms, no jitter",
        poisson_cfg(PoissonDelay::Cap(Duration::from_millis(20)), 0.98, Duration::ZERO),
    );
}
