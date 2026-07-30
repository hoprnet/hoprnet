// OFAT (one-factor-at-a-time) cluster throughput matrices.
//
// Runs a real-QUIC multi-node cluster (mock Blokli chain) and varies one
// dimension (latency shim, write strategy, SURB buffer size, rate control)
// across 1/2/3 hops at a fixed 20 MB baseline.  The *signal* is the printed
// throughput series: a variant that stalls, shows a cliff, or produces higher
// loss is the finding.
//
// All tests are `#[ignore]`d by default — cluster bootstrap takes 60–120 s.
// Run the full suite:
//
//   cargo nextest run -p hopr-lib --features session-client \
//       --test 'cluster_throughput-matrix' -j 1 --run-ignored all -- --no-capture
//
// Run a single variant (e.g. latency 1-hop with 50ms, case index 1):
//
//   cargo nextest run -p hopr-lib --features session-client \
//       --test 'cluster_throughput-matrix' -j 1 --run-ignored all \
//       -- 'latency_matrix::case_1' --no-capture

#![cfg(feature = "session-client")]

use std::time::Duration;

use hopr_lib::{
    SurbBalancerConfig,
    config::TransitLatencyConfig,
    testing::{
        fixtures::{STRESS_WIN_PROB, TEST_GLOBAL_TIMEOUT, stress_cluster_fixture, stress_cluster_fixture_with_latency},
        loadgen::{StressConfig, WriteStrategy, run_stress},
    },
};
use rstest::*;
use serial_test::serial;

/// Bytes transferred per matrix case.  20 MB is long enough to saturate the pipeline
/// and produce meaningful throughput samples, yet short enough to fit inside
/// `TEST_GLOBAL_TIMEOUT` even after the ~100 s cluster bootstrap.
const MATRIX_BYTES: u64 = 20 * 1024 * 1024; // 20 MB

// ── Latency matrix ────────────────────────────────────────────────────────────
//
// Compares loopback (0 ms) vs simulated WAN (50 ms) transit latency per hop.
// The limiter + SURB replenishment round-trip slows under latency — this matrix
// shows *how much* and at which hop count the session stalls.
//
// Expected: 50 ms cases show significantly lower throughput; 3-hop 50 ms may stall
// because each SURB reply also traverses 3 hops × 50 ms = 150 ms one-way round-trip.

#[rstest]
// (hops, latency_ms): 0 = loopback baseline, 50 = WAN simulation
#[case(1, 0)]
#[case(1, 50)]
#[case(2, 0)]
#[case(2, 50)]
#[case(3, 0)]
#[case(3, 50)]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[serial]
#[ignore = "slow: requires cluster bootstrap (60–120 s); run with --run-ignored"]
async fn latency_matrix(#[case] hops: usize, #[case] latency_ms: u64) -> anyhow::Result<()> {
    let n_nodes = hops + 2;
    let cluster = if latency_ms == 0 {
        stress_cluster_fixture(STRESS_WIN_PROB, n_nodes)
    } else {
        let latency = TransitLatencyConfig {
            mean: Duration::from_millis(latency_ms),
            std_dev: Duration::from_millis(latency_ms / 10),
        };
        stress_cluster_fixture_with_latency(STRESS_WIN_PROB, n_nodes, latency)
    };

    let cfg = StressConfig {
        hops,
        total_bytes: MATRIX_BYTES,
        routes: 1,
        ..StressConfig::default()
    };

    let report = run_stress(&cluster, &cfg).await?;
    report.print_series();

    anyhow::ensure!(
        report.total_bytes_delivered >= cfg.total_bytes,
        "delivered {} bytes, expected at least {}",
        report.total_bytes_delivered,
        cfg.total_bytes,
    );
    anyhow::ensure!(!report.samples.is_empty(), "no throughput samples recorded");
    anyhow::ensure!(
        report.samples.iter().any(|s| s.recv_window_bytes > 0),
        "no bytes received — pipeline delivered nothing"
    );

    Ok(())
}

// ── Write-batching matrix ─────────────────────────────────────────────────────
//
// Compares continuous writes (back-to-back, matching the edge-client) vs batched
// writes with a 50 ms pause between 64 KB bursts.
//
// Expected: `Continuous` may show encode-timeout drops (visible in the report) when
// the Rayon pool saturates; `Batched` should deliver 0 drops at the cost of lower peak
// throughput.  The throughput difference quantifies the encode-pool overhead tax.

#[rstest]
#[case(1, WriteStrategy::Continuous)]
#[case(1, WriteStrategy::Batched { batch_bytes: 65536, pause: Duration::from_millis(50) })]
#[case(2, WriteStrategy::Continuous)]
#[case(2, WriteStrategy::Batched { batch_bytes: 65536, pause: Duration::from_millis(50) })]
#[case(3, WriteStrategy::Continuous)]
#[case(3, WriteStrategy::Batched { batch_bytes: 65536, pause: Duration::from_millis(50) })]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[serial]
#[ignore = "slow: requires cluster bootstrap (60–120 s); run with --run-ignored"]
async fn write_batching_matrix(#[case] hops: usize, #[case] strategy: WriteStrategy) -> anyhow::Result<()> {
    let n_nodes = hops + 2;
    let cluster = stress_cluster_fixture(STRESS_WIN_PROB, n_nodes);

    let cfg = StressConfig {
        hops,
        total_bytes: MATRIX_BYTES,
        routes: 1,
        write_strategy: strategy,
        ..StressConfig::default()
    };

    let report = run_stress(&cluster, &cfg).await?;
    report.print_series();

    anyhow::ensure!(
        report.total_bytes_delivered >= cfg.total_bytes,
        "delivered {} bytes, expected at least {}",
        report.total_bytes_delivered,
        cfg.total_bytes,
    );
    anyhow::ensure!(!report.samples.is_empty(), "no throughput samples recorded");
    anyhow::ensure!(
        report.samples.iter().any(|s| s.recv_window_bytes > 0),
        "no bytes received — pipeline delivered nothing"
    );

    Ok(())
}

// ── SURB buffer matrix ────────────────────────────────────────────────────────
//
// Compares the default SURB balancer target (7000) vs a reduced target (1000).
//
// The SURB target controls the steady-state SURB buffer at the Exit.  A smaller
// buffer means SURBs replenish less aggressively and the exit-side rate limiter
// stays throttled longer.  A larger buffer means more keep-alive infrastructure
// traffic but a warmer SURB supply for the Exit's egress rate limiter to ramp.
//
// Expected: smaller target → lower throughput, higher latency variance.

#[rstest]
#[case(1, 7000, 5000)] // default
#[case(1, 1000, 500)] // reduced
#[case(2, 7000, 5000)]
#[case(2, 1000, 500)]
#[case(3, 7000, 5000)]
#[case(3, 1000, 500)]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[serial]
#[ignore = "slow: requires cluster bootstrap (60–120 s); run with --run-ignored"]
async fn surb_buffer_matrix(#[case] hops: usize, #[case] target: u64, #[case] max_per_sec: u64) -> anyhow::Result<()> {
    let n_nodes = hops + 2;
    let cluster = stress_cluster_fixture(STRESS_WIN_PROB, n_nodes);

    let surb_cfg = SurbBalancerConfig {
        target_surb_buffer_size: target,
        max_surbs_per_sec: max_per_sec,
        ..SurbBalancerConfig::default()
    };

    let cfg = StressConfig {
        hops,
        total_bytes: MATRIX_BYTES,
        routes: 1,
        surb_config: Some(surb_cfg),
        ..StressConfig::default()
    };

    let report = run_stress(&cluster, &cfg).await?;
    report.print_series();

    anyhow::ensure!(
        report.total_bytes_delivered >= cfg.total_bytes,
        "delivered {} bytes, expected at least {}",
        report.total_bytes_delivered,
        cfg.total_bytes,
    );
    anyhow::ensure!(!report.samples.is_empty(), "no throughput samples recorded");
    anyhow::ensure!(
        report.samples.iter().any(|s| s.recv_window_bytes > 0),
        "no bytes received — pipeline delivered nothing"
    );

    Ok(())
}

// ── Rate-control matrix ───────────────────────────────────────────────────────
//
// Compares the exit-side egress rate limiter on (default, 10 pkt/s cold-start +
// balancer ramp) vs off (NoRateControl).
//
// The limiter matches Exit reply throughput to the SURB replenishment rate.  At
// loopback speeds SURBs replenish fast enough that the limiter rarely throttles.
// The gap between on/off quantifies the cold-start tax and shows whether the
// balancer ramps quickly enough under local-cluster conditions.
//
// Expected: `rate_control=off` (NoRateControl) achieves higher throughput,
// especially early in the run before the balancer has ramped; 3-hop gap may be
// larger because SURB round-trips are longer.

#[rstest]
#[case(1, true)] // limiter on  (production default)
#[case(1, false)] // limiter off (NoRateControl)
#[case(2, true)]
#[case(2, false)]
#[case(3, true)]
#[case(3, false)]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[serial]
#[ignore = "slow: requires cluster bootstrap (60–120 s); run with --run-ignored"]
async fn rate_control_matrix(#[case] hops: usize, #[case] rate_control: bool) -> anyhow::Result<()> {
    let n_nodes = hops + 2;
    let cluster = stress_cluster_fixture(STRESS_WIN_PROB, n_nodes);

    let cfg = StressConfig {
        hops,
        total_bytes: MATRIX_BYTES,
        routes: 1,
        rate_control,
        ..StressConfig::default()
    };

    let report = run_stress(&cluster, &cfg).await?;
    report.print_series();

    anyhow::ensure!(
        report.total_bytes_delivered >= cfg.total_bytes,
        "delivered {} bytes, expected at least {}",
        report.total_bytes_delivered,
        cfg.total_bytes,
    );
    anyhow::ensure!(!report.samples.is_empty(), "no throughput samples recorded");
    anyhow::ensure!(
        report.samples.iter().any(|s| s.recv_window_bytes > 0),
        "no bytes received — pipeline delivered nothing"
    );

    Ok(())
}
