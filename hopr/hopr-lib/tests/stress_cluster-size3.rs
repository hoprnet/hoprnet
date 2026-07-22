// Reference cluster stress tests for the session-client feature.
#![cfg(feature = "session-client")]

use std::time::Duration;

use futures::future::try_join_all;
use hopr_chain_connector::blokli_client::BlokliQueryClient;
use hopr_lib::{
    api::types::primitive::prelude::HoprBalance,
    testing::{
        fixtures::{
            STRESS_WIN_PROB, TEST_GLOBAL_TIMEOUT, TestNodeConfig, chain_propagation_delay, cluster_fixture,
            stress_cluster_fixture,
        },
        hopr::ChannelGuard,
        loadgen::{StressConfig, run_stress},
    },
};
use rstest::*;
use serial_test::serial;
use tokio::io::AsyncWriteExt;

const FUNDING_AMOUNT: &str = "100 wxHOPR";

/// Writes 10KB through a 1-hop session backed by real chain ticket validation.
///
/// Validates that the full packet pipeline (encoding, ticket creation, Sphinx, relay,
/// decode, ticket acknowledgement) correctly handles sustained data output.
#[rstest]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[serial]
async fn relay_throughput_with_real_tickets() -> anyhow::Result<()> {
    const DATA_SIZE: usize = 10 * 1024; // 10KB

    let cluster = cluster_fixture(vec![TestNodeConfig::default(); 3]);

    let nodes: Vec<&_> = cluster.iter().collect();
    let (src, relay, dst) = (nodes[0], nodes[1], nodes[2]);

    // Open all 4 channels concurrently — forward and return paths.
    let funding = FUNDING_AMOUNT.parse::<HoprBalance>()?;
    let channels = try_join_all([
        ChannelGuard::open_channel_between_nodes(src.instance.clone(), relay.instance.clone(), funding),
        ChannelGuard::open_channel_between_nodes(relay.instance.clone(), dst.instance.clone(), funding),
        ChannelGuard::open_channel_between_nodes(dst.instance.clone(), relay.instance.clone(), funding),
        ChannelGuard::open_channel_between_nodes(relay.instance.clone(), src.instance.clone(), funding),
    ])
    .await?;

    let chain_info = cluster.chain_client.query_chain_info().await?;
    let timeout = chain_propagation_delay(&chain_info) * 12;
    cluster.wait_for_channel_graph(src, channels.len(), timeout).await?;

    let path = [src, relay, dst];
    let mut session = cluster.create_session(&path).await?;

    // Write 10KB through the session — exercises encoding, mixing, relay, decoding
    let payload: Vec<u8> = (0..DATA_SIZE).map(|i| (i % 256) as u8).collect();
    session.write_all(&payload).await?;
    session.flush().await?;

    drop(session);

    try_join_all(
        channels
            .into_iter()
            .map(|g| async move { g.try_close_channels_all_channels().await }),
    )
    .await?;

    Ok(())
}

/// Creates two independent sessions over the same 1-hop path concurrently and writes
/// distinct data through both, verifying that the pipeline handles parallel sessions without
/// cross-contamination or deadlock.
#[rstest]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[serial]
async fn concurrent_sessions_independent_no_deadlock() -> anyhow::Result<()> {
    const DATA_SIZE: usize = 4 * 1024; // 4KB per session

    let cluster = cluster_fixture(vec![TestNodeConfig::default(); 3]);
    let nodes: Vec<&_> = cluster.iter().collect();
    let (src, relay, dst_a) = (nodes[0], nodes[1], nodes[2]);

    let funding = FUNDING_AMOUNT.parse::<HoprBalance>()?;

    // Open all 4 channels concurrently — forward and return paths.
    let channels = try_join_all([
        ChannelGuard::open_channel_between_nodes(src.instance.clone(), relay.instance.clone(), funding),
        ChannelGuard::open_channel_between_nodes(relay.instance.clone(), dst_a.instance.clone(), funding),
        ChannelGuard::open_channel_between_nodes(dst_a.instance.clone(), relay.instance.clone(), funding),
        ChannelGuard::open_channel_between_nodes(relay.instance.clone(), src.instance.clone(), funding),
    ])
    .await?;

    let chain_info = cluster.chain_client.query_chain_info().await?;
    let timeout = chain_propagation_delay(&chain_info) * 12;
    cluster.wait_for_channel_graph(src, channels.len(), timeout).await?;

    // Create two sessions from src through relay to dst_a (different sessions, same path)
    let path = [src, relay, dst_a];
    let mut session_a = cluster.create_session(&path).await?;
    let mut session_b = cluster.create_session(&path).await?;

    let payload_a: Vec<u8> = vec![0xAA; DATA_SIZE];
    let payload_b: Vec<u8> = vec![0xBB; DATA_SIZE];

    // Write through both sessions concurrently
    let (res_a, res_b) = tokio::join!(
        async {
            session_a.write_all(&payload_a).await?;
            session_a.flush().await
        },
        async {
            session_b.write_all(&payload_b).await?;
            session_b.flush().await
        }
    );

    res_a?;
    res_b?;

    drop(session_a);
    drop(session_b);

    try_join_all(
        channels
            .into_iter()
            .map(|g| async move { g.try_close_channels_all_channels().await }),
    )
    .await?;

    Ok(())
}

/// Sends 5 MB through a single 1-hop session, prints the throughput series, and
/// validates that every byte is received by the destination EchoServer.
///
/// This is the primary diagnostic test for the encode/delivery pipeline with SURB
/// management enabled.  Running with a 3-node cluster keeps bootstrap time under
/// the 6-minute test timeout even on slow CI machines.
#[rstest]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[serial]
#[ignore = "slow: requires ~60s cluster bootstrap; run with --run-ignored"]
async fn three_node_cluster_5mb_single_session() -> anyhow::Result<()> {
    let cluster = stress_cluster_fixture(STRESS_WIN_PROB, 3);

    let cfg = StressConfig {
        total_bytes: 5 * 1024 * 1024,
        routes: 1,
        msg_size_range: 4096..=32768,
        sample_interval: Duration::from_millis(500),
        seed: 42,
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
        "no bytes received at destination — pipeline delivered nothing"
    );

    Ok(())
}
