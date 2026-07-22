// 5-node cluster throughput smoke test.
//
// This test is `#[ignore]`d by default — it takes several minutes due to
// cluster bootstrap time (~100 s) plus the 5 MB transfer.  Run it explicitly:
//
//   cargo nextest run -p hopr-lib --features testing \
//       --test 'stress_cluster-size5' -j 1 --run-ignored all -- --nocapture
//
// For a profiling run use the `stress_cluster` example binary instead, which
// gives a clean single entry point for `cargo flamegraph` / `perf`.

#![cfg(feature = "session-client")]

use std::time::Duration;

use hopr_lib::testing::{
    fixtures::{TEST_GLOBAL_TIMEOUT, TestNodeConfig, cluster_fixture},
    loadgen::{StressConfig, run_stress},
};
use rstest::*;
use serial_test::serial;

/// Sends 5 MB of 1-hop traffic across 2 concurrent sessions on a 5-node
/// cluster and validates basic throughput liveness.
///
/// The test uses a conservative volume (5 MB) and only 2 sessions so it
/// comfortably fits within `TEST_GLOBAL_TIMEOUT` even after the long cluster
/// bootstrap.  For real load testing use the `stress_cluster` example binary.
#[rstest]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[serial]
#[ignore = "slow: requires ~100 s cluster bootstrap; run with --run-ignored"]
async fn five_node_cluster_should_sustain_throughput_over_1hop_sessions() -> anyhow::Result<()> {
    let cluster = cluster_fixture(vec![TestNodeConfig::default(); 5]);

    let cfg = StressConfig {
        total_bytes: 5 * 1024 * 1024, // 5 MB — completes well inside TEST_GLOBAL_TIMEOUT
        hops: 1,
        routes: 2,
        msg_size_range: 4096..=32768,
        sample_interval: Duration::from_millis(500),
        seed: 42,
        ..StressConfig::default()
    };

    let report = run_stress(&cluster, &cfg).await?;

    report.print_series();

    // ── Validation criteria ────────────────────────────────────────────────────
    //
    // We assert liveness, not performance: no hard MB/s threshold is set
    // because the mock chain and coverage instrumentation introduce variable
    // overhead.  The real signal is the printed series and a flame graph from
    // the example binary.

    anyhow::ensure!(
        report.total_bytes_delivered >= cfg.total_bytes,
        "delivered {} bytes, expected at least {}",
        report.total_bytes_delivered,
        cfg.total_bytes,
    );

    anyhow::ensure!(!report.samples.is_empty(), "no throughput samples were recorded");

    // Throughput must be positive in at least one window — the pipeline should
    // never be completely stalled for the full duration.
    let any_nonzero = report.samples.iter().any(|s| s.window_bytes > 0);
    anyhow::ensure!(
        any_nonzero,
        "all throughput windows recorded zero bytes — pipeline appears stalled"
    );

    Ok(())
}
