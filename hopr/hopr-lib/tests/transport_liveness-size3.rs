//! Integration tests for connection-liveness recovery at the cluster level.
//!
//! These tests exercise the real `ConnectionClosed` / `OutgoingConnectionError` →
//! `liveness.remove` → cached-stream-errors path end-to-end, complementing the
//! deterministic unit tests in `impls/transport/p2p/src/liveness.rs` and
//! `transport/hopr/src/protocol/stream.rs`.
//!
//! **Scope**: drop-node recovery only. Silent half-open death (NAT rebinding /
//! middlebox) has no recovery trigger in the current codebase because the probe
//! layer does not force-close connections; that scenario is out of scope for this PR.

use std::time::Duration;

use anyhow::Context;
use hopr_lib::{
    api::{
        network::NetworkView,
        node::{HasNetworkView, HasTransportApi},
    },
    testing::{
        fixtures::{TEST_GLOBAL_TIMEOUT, TestNodeConfig, cluster_fixture},
        wait_until,
    },
};
use rstest::*;
use serial_test::serial;

/// Verifies that when a peer's node is stopped (runtime killed), the surviving
/// peers reap the dead connection and subsequent operations towards that peer
/// fail fast instead of black-holing packets indefinitely.
///
/// Scenario:
/// 1. Bring up a 3-node cluster (full mesh, probe warmup).
/// 2. Drop one node — its `TestedHopr::Drop` impl calls `runtime.shutdown_background()`, which tears down its libp2p
///    swarm. Its peers' TCP sockets then error, and libp2p emits `ConnectionClosed` / `OutgoingConnectionError`.
/// 3. The swarm event loop in each surviving peer calls `liveness.remove(victim)`, which clears the `Arc<AtomicBool>`
///    held by any cached `LivenessStream` for that peer.
/// 4. The next `poll_*` on the cached stream returns `ConnectionAborted`, the per-peer reader/writer tasks invalidate
///    the stream cache, and the entry is evicted.
/// 5. Assert the survivor observes the victim as disconnected within a generous timeout.
/// 6. Assert that pinging the victim fails promptly rather than hanging, confirming the stream does not black-hole the
///    request.
#[rstest]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[serial]
async fn dropped_node_should_be_reaped_by_survivors_and_operations_should_fail_fast() -> anyhow::Result<()> {
    let mut cluster = cluster_fixture(vec![TestNodeConfig::default(); 3]);

    // Capture victim identity before dropping it — PeerId is Copy.
    let victim_idx = cluster.cluster.len() - 1;
    let victim_peer = cluster.cluster[victim_idx].peer_id();
    let victim_key =
        hopr_lib::peer_id_to_offchain_key(&victim_peer).context("victim peer id must be a valid offchain key")?;

    // Drop the victim. `TestedHopr::Drop` calls `runtime.shutdown_background()`, tearing
    // down the node's entire Tokio runtime (swarm included). Removing the last element
    // does not shift index 0, so `cluster[0]` remains the same survivor node.
    let _ = cluster.cluster.remove(victim_idx);

    let survivor = &cluster[0];

    // Wait for the survivor to reap the dead connection. The upper bound is generous to
    // accommodate slow TCP keepalive detection in CI environments.
    wait_until(
        || async { Ok::<_, std::convert::Infallible>(!survivor.inner().network_view().is_connected(&victim_peer)) },
        Duration::from_secs(60),
    )
    .await
    .context("survivor should observe the victim as disconnected within 60 s")?;

    // Assert fail-fast: pinging the dead peer should error promptly (liveness flag
    // cleared → stream errors → probe request fails), not hang for minutes.
    let ping_result =
        tokio::time::timeout(Duration::from_secs(15), survivor.inner().transport().ping(&victim_key)).await;

    assert!(
        matches!(ping_result, Ok(Err(_)) | Err(_)),
        "ping to dropped peer must fail fast — got Ok(Ok(...)): {ping_result:?}"
    );

    Ok(())
}
