//! End-to-end verification that the CPU-scaled packet-pipeline concurrency defaults fix the 4.1.x
//! relay throughput regression without starving the exit or the client.
//!
//! Background: #8246 capped the ingress decode-concurrency default at `pool_thread_count - 2`.
//! Because the production Rayon pool is only `available_parallelism()/2`, that collapsed decode to
//! `1` on small hosts, halving relay forwarding. The fix makes the default a CPU-derived deep queue
//! (`available_parallelism * 8`) instead. These tests drive real-QUIC multi-hop traffic (mock chain)
//! and assert the fix holds end to end.
#![cfg(feature = "session-client")]

use std::time::Duration;

use hopr_lib::{
    exports::transport::protocol::PacketPipelineConfig,
    testing::{
        fixtures::{
            STRESS_WIN_PROB, TEST_GLOBAL_TIMEOUT, TestNodeConfig, stress_cluster_fixture,
            stress_cluster_fixture_with_configs,
        },
        loadgen::{StressConfig, run_stress},
    },
};
use rstest::*;
use serial_test::serial;

/// Corrected (CPU-scaled) defaults must keep BOTH the forwarding relays and the terminating exit
/// healthy under sustained bidirectional load: the volume is delivered and neither the decode path
/// (relay/exit ingress) nor the encode path (exit/client SURB + data egress) sheds packets to a
/// Rayon timeout. A starved decode or encode stage — the old `pool - 2` collapse, or a
/// deep-encode-queue regression — would surface here as timeout drops.
#[rstest]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[serial]
#[ignore = "slow: requires cluster bootstrap (60–120 s); run with --run-ignored"]
async fn corrected_defaults_keep_relay_and_exit_healthy() -> anyhow::Result<()> {
    // 2 hops => src → r1 → r2 → exit: genuine forwarding relays AND a distinct terminating exit.
    const HOPS: usize = 2;
    let cluster = stress_cluster_fixture(STRESS_WIN_PROB, HOPS + 2);

    let cfg = StressConfig {
        hops: HOPS,
        total_bytes: 20 * 1024 * 1024,
        routes: 1,
        msg_size_range: 4096..=32768,
        sample_interval: Duration::from_millis(500),
        seed: 42,
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
    anyhow::ensure!(
        report.samples.iter().any(|s| s.recv_window_bytes > 0),
        "no bytes received at destination — pipeline delivered nothing",
    );
    // The load-bearing assertions: no stage starved.
    anyhow::ensure!(
        report.decode_timeout_drops == 0,
        "decode path shed {} packets to a Rayon timeout — the relay/exit ingress is starved",
        report.decode_timeout_drops,
    );
    anyhow::ensure!(
        report.encode_timeout_drops == 0,
        "encode path shed {} packets to a Rayon timeout — the exit/client SURB+data egress is starved",
        report.encode_timeout_drops,
    );

    Ok(())
}

/// Contrast run demonstrating the fix actually governs throughput: pinning every node's ingress
/// decode concurrency to `1` (reproducing the collapsed pre-fix default) must measurably reduce
/// *delivery* throughput versus the CPU-scaled default over the identical workload.
///
/// The meaningful metric is received (delivered) MB/s, not sent MB/s: a decode bottleneck lets the
/// source keep writing into local session buffers while packets stall downstream, so the send rate
/// can even rise while delivery collapses — which is exactly the report's downstream "frame expired"
/// symptom. Compared with a generous margin rather than an absolute floor, since the mock chain and
/// coverage instrumentation make absolute rates machine-dependent.
#[rstest]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[serial]
#[ignore = "slow: boots two clusters (120–240 s); run with --run-ignored"]
async fn pinned_low_decode_concurrency_degrades_throughput() -> anyhow::Result<()> {
    const HOPS: usize = 2;
    const N: usize = HOPS + 2;

    let cfg = StressConfig {
        hops: HOPS,
        total_bytes: 20 * 1024 * 1024,
        routes: 1,
        msg_size_range: 4096..=32768,
        sample_interval: Duration::from_millis(500),
        seed: 42,
        ..StressConfig::default()
    };

    // Baseline: CPU-scaled defaults on every node.
    let default_recv_mbps = {
        let cluster = stress_cluster_fixture(STRESS_WIN_PROB, N);
        let report = run_stress(&cluster, &cfg).await?;
        report.print_series();
        report.mean_recv_mbps
    };

    // Collapsed: pin decode concurrency to 1 on every node, mirroring the pre-fix `pool - 2` result.
    let pinned_recv_mbps = {
        let configs = (0..N)
            .map(|_| TestNodeConfig {
                win_prob: STRESS_WIN_PROB,
                pipeline: Some(PacketPipelineConfig {
                    input_concurrency: Some(1),
                    ..Default::default()
                }),
                ..TestNodeConfig::default()
            })
            .collect();
        let cluster = stress_cluster_fixture_with_configs(configs);
        let report = run_stress(&cluster, &cfg).await?;
        report.print_series();
        report.mean_recv_mbps
    };

    anyhow::ensure!(
        default_recv_mbps > pinned_recv_mbps * 2.0,
        "CPU-scaled default delivery ({default_recv_mbps:.2} MB/s) should clearly beat pinned decode=1 \
         ({pinned_recv_mbps:.2} MB/s); the concurrency default is not governing delivery throughput",
    );

    Ok(())
}
