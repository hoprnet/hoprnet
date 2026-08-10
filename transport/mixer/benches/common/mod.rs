//! Shared helpers for the mixer throughput benchmarks (`mixer_throughput_bench`,
//! `poisson_shared_bench`), so the workload volumes, labels, payload and reused-channel drain
//! are defined once. Included via `mod common;` in each bench (compiled per bench binary), so a
//! bench that uses only a subset would warn — every item here is used by both current benches.

use std::{cell::RefCell, rc::Rc};

use futures::{StreamExt, future::poll_fn};
use hopr_transport_mixer::config::{MixerConfig, MixerType, PoissonConfig};

pub const SAMPLE_SIZE: usize = 10;

/// 512 characters long string of random gibberish.
pub const RANDOM_GIBBERISH: &str = "abcdferjskdiq7LGuzjfXMEI2tTCUIZsCDsHnfycUbPcA1boJ48Jm7xBBNIvxsrbK3bNCevOMXYMqrhsVBXfmKy23K7ItgbuObTmqk0ndfceAhugLZveAhp4Xx1vHCAROY69sOTJiia3EBC2aXSBpUfb3WHSJDxHRMHwzCwd0BPj4WFi4Ig884Ph6altlFWzpL3ILsHmLxy9KoPCAtolb3YEegMCI4y9BsoWyCtcZdBHBrqXaSzuJivw5J1DBudj3Z6oORrEfRuFIQLi0l89Emc35WhSyzOdguC1x9PS8AiIAu7UoXlp3VIaqVUu4XGUZ21ABxI9DyMzxGbOOlsrRGFFN9G8di9hqIX1UOZpRgMNmtDwZoyoU2nGLoWGM58buwuvbNkLjGu2X9HamiiDsRIR4vxi5i61wIP6VueVOb68wvbz8csR88OhFsExjGBD9XXtJvUjy1nwdkikBOblNm2FUbyq8aHwHocoMqZk8elbYMHgbjme9d1CxZQKRwOR";

/// A near-passthrough config (1 ms cap) so the benchmark measures the channel's per-message
/// overhead rather than the mixing delay. The `Poisson` variant is read by both Poisson engines
/// (via `PoissonParams::from_mixer`); the uniform channel sees no `Uniform` config and so applies
/// zero delay — equally minimal for a throughput measurement.
#[inline]
pub fn minimal_delay_mixer_cfg() -> MixerConfig {
    MixerConfig {
        mixer_type: MixerType::Poisson(PoissonConfig {
            max_cap: std::time::Duration::from_millis(1),
            ..PoissonConfig::default()
        }),
        ..MixerConfig::default()
    }
}

/// Workload volumes spanning the realistic 1–10 MB/s operating range. Each volume is one second
/// of offered load at 1, 5 and 10 MB/s respectively, so the reported throughput shows how much
/// headroom the engine has over the target rate.
pub fn sizes() -> [usize; 3] {
    [1_000_000, 5_000_000, 10_000_000]
}

/// e.g. "1_MB_per_s" — the volume equals one second of load at this rate.
pub fn size_label(bytes: usize) -> String {
    format!("{}_MB_per_s", bytes / 1_000_000)
}

/// Drain one item, borrowing the shared receiver only inside each poll (never across the
/// `.await`) so the `Rc<RefCell<_>>` handle can be owned by the future without tripping
/// `clippy::await_holding_refcell_ref`.
pub async fn drain_one<R: StreamExt + Unpin>(rx: &Rc<RefCell<R>>) -> Option<R::Item> {
    poll_fn(|cx| rx.borrow_mut().poll_next_unpin(cx)).await
}
