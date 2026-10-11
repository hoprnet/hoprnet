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

/// Verifies that an exit stops replying through a relay that went offline: once the connection to the relay
/// is gone, the exit holds no SURB whose return path starts with it (they would only lose the replies), and
/// marks the relay unreachable so it stores no new ones through it.
///
/// Scenario:
/// 1. A 3-node cluster `src -> relay -> dst`, with channels `src -> relay` (forward) and `dst -> relay` (return path).
/// 2. A 1-hop session from `src` to `dst` with the SURB balancer on, until `dst` holds SURBs through `relay`.
/// 3. Drop `relay` (its runtime shuts down) and wait until `dst` sees it disconnected.
/// 4. Assert `dst` marks it unreachable and holds no SURB through it.
#[rstest]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[serial]
#[cfg(feature = "session-client")]
async fn exit_should_purge_surbs_through_a_relay_that_disconnected() -> anyhow::Result<()> {
    use hopr_lib::{
        api::{
            chain::{ChainKeyOperations, KeyIdMapping},
            node::HasChainApi,
            types::primitive::prelude::HoprBalance,
        },
        testing::hopr::ChannelGuard,
    };

    let mut cluster = cluster_fixture(vec![TestNodeConfig::default(); 3]);
    let (src_idx, relay_idx, dst_idx) = (0, 1, 2);

    let relay_peer = cluster.cluster[relay_idx].peer_id();
    let relay_key =
        hopr_lib::peer_id_to_offchain_key(&relay_peer).context("relay peer id must be a valid offchain key")?;
    let relayer = cluster.cluster[dst_idx]
        .inner()
        .chain_api()
        .key_id_mapper_ref()
        .map_key_to_id(&relay_key)
        .context("the exit must know the relay's key id")?;

    let funding = "100 wxHOPR".parse::<HoprBalance>()?;
    let _channels = [
        ChannelGuard::open_channel_between_nodes(
            cluster.cluster[src_idx].instance.clone(),
            cluster.cluster[relay_idx].instance.clone(),
            funding,
        )
        .await?,
        ChannelGuard::open_channel_between_nodes(
            cluster.cluster[dst_idx].instance.clone(),
            cluster.cluster[relay_idx].instance.clone(),
            funding,
        )
        .await?,
    ];
    cluster
        .wait_for_channel_graph(&cluster.cluster[src_idx], 2, Duration::from_secs(60))
        .await?;
    cluster
        .wait_for_channel_graph(&cluster.cluster[dst_idx], 2, Duration::from_secs(60))
        .await?;

    let _session = cluster
        .create_session(&[
            &cluster.cluster[src_idx],
            &cluster.cluster[relay_idx],
            &cluster.cluster[dst_idx],
        ])
        .await?;

    let held_through_relay = || {
        cluster.cluster[dst_idx]
            .inner()
            .transport()
            .surb_store()
            .count_surbs_through(&relayer)
    };
    wait_until(
        || async { Ok::<_, std::convert::Infallible>(held_through_relay() > 0) },
        Duration::from_secs(60),
    )
    .await
    .context("the exit should receive SURBs through the relay from the session's SURB balancer")?;
    tracing::info!(held = held_through_relay(), "exit holds SURBs through the relay");

    // Drop the relay; the exit is now at index 1.
    drop(cluster.cluster.remove(relay_idx));
    let dst = &cluster.cluster[1];

    wait_until(
        || async { Ok::<_, std::convert::Infallible>(!dst.inner().network_view().is_connected(&relay_peer)) },
        Duration::from_secs(60),
    )
    .await
    .context("the exit should observe the relay as disconnected within 60 s")?;

    let store = dst.inner().transport().surb_store();
    wait_until(
        || async {
            Ok::<_, std::convert::Infallible>(
                store.is_relayer_unreachable(&relayer) && store.count_surbs_through(&relayer) == 0,
            )
        },
        Duration::from_secs(10),
    )
    .await
    .with_context(|| {
        format!(
            "the exit should purge SURBs through the disconnected relay: unreachable={} still held={}",
            store.is_relayer_unreachable(&relayer),
            store.count_surbs_through(&relayer)
        )
    })?;

    Ok(())
}
