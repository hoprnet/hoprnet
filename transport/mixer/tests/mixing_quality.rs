//! End-to-end mixing-quality checks for the exponential (Poisson) release engine.
//!
//! Drives a steady packet stream through the assembled `poisson_channel` (dedicated thread +
//! adaptive timer) and asserts the realized end-to-end delay distribution matches the configured
//! anchor: exponential mean, cap-truncation fraction, and that reordering actually happens. These
//! are the enforced counterparts of the earlier throwaway measurement scenarios — proofs that
//! must hold, not numbers to eyeball.
//!
//! Assertions are deliberately loose (wide absolute bands plus robust cross-scenario ordering) so
//! they enforce the design intent without flaking on OS-scheduling jitter in the realized delays.

#![cfg(feature = "poisson")]

use std::time::{Duration, Instant};

use futures::StreamExt;
use futures_timer::Delay;
use hopr_transport_mixer::{MixerConfig, MixerType, PoissonConfig, PoissonDelay, poisson_channel};

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

/// Realized-delay statistics measured end-to-end through the channel.
struct Stats {
    cap_ms: f64,
    observed_mean: f64,
    p99: f64,
    /// Fraction of packets whose realized delay landed at or above the cap.
    at_or_over_cap_frac: f64,
    /// Fraction of packets that arrived out of send order (evidence of mixing).
    out_of_order_frac: f64,
}

fn run_scenario(cfg: MixerConfig) -> Stats {
    let cap_ms = match cfg.mixer_type {
        MixerType::Poisson(pc) => pc.delay.resolve(pc.cap_percentile).0.as_secs_f64() * 1000.0,
        #[allow(unreachable_patterns)]
        _ => unreachable!("scenario configs are always the dedicated-thread Poisson variant"),
    };

    let (mut delays_ms, out_of_order) = futures::executor::block_on(async move {
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
    assert_eq!(n, COUNT, "every enqueued packet must be delivered before close");

    // Sum and cap-count are order-independent; fold them in one pass, then sort in place for the
    // percentile (no second copy of the delay vector needed).
    let (sum, at_or_over_cap) = delays_ms
        .iter()
        .fold((0.0, 0usize), |(s, c), d| (s + d, c + (*d >= cap_ms - 1e-6) as usize));
    delays_ms.sort_by(|a, b| a.partial_cmp(b).unwrap());

    Stats {
        cap_ms,
        observed_mean: sum / n as f64,
        p99: percentile(&delays_ms, 0.99),
        at_or_over_cap_frac: at_or_over_cap as f64 / n as f64,
        out_of_order_frac: out_of_order as f64 / n as f64,
    }
}

/// The realized end-to-end delay distribution must track the configured anchor across the three
/// canonical regimes, and mixing (reordering) must actually occur.
#[test]
fn poisson_channel_delay_distribution_should_match_configuration() {
    let mean = Duration::from_millis(10);
    let jitter = Duration::from_millis(2);

    // Mean 10 ms, cap ~20 ms (86.5th percentile): the baseline regime.
    let tight = run_scenario(poisson_cfg(PoissonDelay::Mean(mean), 0.865, jitter));
    // Mean 10 ms, cap ~100 ms (99.995th percentile): the cap is far out, so truncation is
    // negligible and the distribution is essentially untruncated exponential.
    let relaxed = run_scenario(poisson_cfg(PoissonDelay::Mean(mean), 0.99995, jitter));
    // Cap 20 ms at the 98th percentile ⇒ mean ~5.11 ms, ~2% force-released. Jitter off so the
    // truncated mass lands exactly at the cap and is countable.
    let capped = run_scenario(poisson_cfg(
        PoissonDelay::Cap(Duration::from_millis(20)),
        0.98,
        Duration::ZERO,
    ));

    // Mixing happens: a steady stream must be reordered on the way out.
    assert!(
        tight.out_of_order_frac > 0.1,
        "expected substantial reordering, got {:.1}%",
        100.0 * tight.out_of_order_frac
    );

    // Observed mean tracks the 10 ms target (wide band absorbs scheduling overhead).
    assert!(
        (5.0..=20.0).contains(&tight.observed_mean),
        "mean-anchored observed mean {:.2} ms outside [5, 20]",
        tight.observed_mean
    );
    assert!(
        (5.0..=20.0).contains(&relaxed.observed_mean),
        "relaxed observed mean {:.2} ms outside [5, 20]",
        relaxed.observed_mean
    );

    // A far-out cap truncates far less than a tight one.
    assert!(
        relaxed.at_or_over_cap_frac < 0.02,
        "relaxed cap should barely truncate, got {:.2}%",
        100.0 * relaxed.at_or_over_cap_frac
    );
    assert!(
        relaxed.at_or_over_cap_frac < tight.at_or_over_cap_frac,
        "relaxed cap ({:.2}%) must truncate less than the tight cap ({:.2}%)",
        100.0 * relaxed.at_or_over_cap_frac,
        100.0 * tight.at_or_over_cap_frac
    );

    // A cap anchored at the 98th percentile derives a smaller mean than the 10 ms anchor, and its
    // force-release fraction sits near the configured 2% (loose band for scheduling noise).
    assert!(
        capped.observed_mean < tight.observed_mean,
        "cap@98% mean {:.2} ms should be below the 10 ms-anchored mean {:.2} ms",
        capped.observed_mean,
        tight.observed_mean
    );
    assert!(
        (0.002..=0.10).contains(&capped.at_or_over_cap_frac),
        "cap@98% force-release fraction {:.2}% outside [0.2%, 10%]",
        100.0 * capped.at_or_over_cap_frac
    );
    // Nothing is delivered meaningfully past the hard cap.
    assert!(
        capped.p99 <= capped.cap_ms + 5.0,
        "p99 {:.2} ms should not exceed cap {:.2} ms by more than 5 ms slack",
        capped.p99,
        capped.cap_ms
    );
}
