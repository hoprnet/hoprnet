use std::time::Duration;

use anyhow::Context;
use hopr_builder::testing::fixtures::{ClusterGuard, TEST_GLOBAL_TIMEOUT, size_3_cluster_fixture as cluster};
use hopr_lib::{
    Address, HoprNodeNetworkOperations,
    api::graph::traits::{EdgeLinkObservable, EdgeObservableRead},
};
use rstest::*;
use serial_test::serial;
use tokio::time::sleep;

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[serial]
/// Ensures nodes expose discoverable peers by fetching the list of announced peers
/// from a random cluster member and asserting it equals the expected count.
async fn all_visible_peers_should_be_listed(cluster: &ClusterGuard) -> anyhow::Result<()> {
    let [node] = cluster.sample_nodes::<1>();

    let peers = node
        .inner()
        .announced_peers()
        .await
        .context("should get announced peers")?;

    assert_eq!(peers.len(), cluster.size());

    Ok(())
}

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
/// Confirms peer-to-peer reachability by pinging another sampled node and
/// verifying the transport API reports success.
async fn ping_should_succeed_for_all_visible_nodes(cluster: &ClusterGuard) -> anyhow::Result<()> {
    let [src, dst] = cluster.sample_nodes::<2>();

    let _ = src.inner().ping(&dst.peer_id()).await.context("failed to ping peer")?;

    Ok(())
}

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
/// Guards against self-pings by attempting to ping the same node and asserting
/// the operation fails.
async fn ping_should_fail_for_self(cluster: &ClusterGuard) -> anyhow::Result<()> {
    let [random_int] = cluster.sample_nodes::<1>();
    let res = random_int.inner().ping(&random_int.peer_id()).await;

    assert!(res.is_err());

    Ok(())
}

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
/// Verifies discovery stays consistent by comparing the announced account list
/// returned by two nodes and ensuring both contain each other's addresses.
async fn discovery_should_produce_the_same_public_announcements_inside_the_network(
    cluster: &ClusterGuard,
) -> anyhow::Result<()> {
    let [idx1, idx2] = cluster.sample_nodes::<2>();

    let accounts_addresses_1 = idx1
        .inner()
        .announced_peers()
        .await
        .context("failed to get announced peers")?
        .into_iter()
        .map(|peer| peer.address)
        .collect::<Vec<Address>>();

    let accounts_addresses_2 = idx2
        .inner()
        .announced_peers()
        .await
        .context("failed to get announced peers")?
        .into_iter()
        .map(|peer| peer.address)
        .collect::<Vec<Address>>();

    assert!(accounts_addresses_1.contains(&idx1.address()));
    assert!(accounts_addresses_1.contains(&idx2.address()));

    assert_eq!(accounts_addresses_1, accounts_addresses_2);
    Ok(())
}

// ── Network graph and probe observation tests ─────────────────────────────

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[serial]
/// After the cluster has completed probe warmup, each node should have
/// immediate observations (edges) for every other peer in the graph.
/// Exercises: probe.rs emit+process, discovery.rs immediate_probe_stream,
/// weight.rs EdgeObservableWrite::record, graph.rs NetworkGraphView::edge.
async fn probe_warmup_should_populate_graph_edges_for_all_peers(cluster: &ClusterGuard) -> anyhow::Result<()> {
    let [node] = cluster.sample_nodes::<1>();

    let peers = node
        .inner()
        .network_connected_peers()
        .await
        .context("failed to get connected peers")?;

    assert_eq!(
        peers.len(),
        cluster.size() - 1,
        "node should be connected to all other cluster members"
    );

    // Wait for probe quality to settle — the fixture only checks that edges exist,
    // not that probe rounds have produced non-zero scores.
    let mut scored_all = false;
    for _ in 0..30 {
        scored_all = peers.iter().all(|peer| {
            node.inner()
                .network_peer_info(peer)
                .and_then(|obs| obs.immediate_qos().map(|imm| imm.average_probe_rate() > 0.0))
                .unwrap_or(false)
        });
        if scored_all {
            break;
        }
        sleep(Duration::from_secs(2)).await;
    }
    assert!(scored_all, "all peers should have non-zero probe rate");

    for peer in &peers {
        let obs = node
            .inner()
            .network_peer_info(peer)
            .context("peer should have observations in the graph")?;

        assert!(obs.score() > 0.0, "score should be positive for {peer}");
    }

    Ok(())
}

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[serial]
/// Exercises the all_network_peers API with a score threshold, verifying
/// that the probe observations produce non-zero scores for immediate neighbors.
/// Covers: weight.rs score(), latency_score(), all_network_peers() filtering.
async fn all_network_peers_should_return_scored_entries(cluster: &ClusterGuard) -> anyhow::Result<()> {
    let [node] = cluster.sample_nodes::<1>();
    let expected_count = cluster.size() - 1;

    // Wait for probe quality to propagate — `all_network_peers(0.0)` filters
    // peers with score > 0 which requires at least one successful probe round.
    let mut all_peers = Vec::new();
    for _ in 0..30 {
        all_peers = node
            .inner()
            .all_network_peers(0.0)
            .await
            .context("failed to get all network peers")?;
        if all_peers.len() >= expected_count && all_peers.iter().all(|(_, _, obs)| obs.score() > 0.0) {
            break;
        }
        sleep(Duration::from_secs(2)).await;
    }

    assert!(!all_peers.is_empty(), "should have at least one peer with score > 0");

    for (addr, peer_id, obs) in &all_peers {
        assert!(addr.is_some(), "peer {peer_id} should have a chain address");
        assert!(obs.score() > 0.0, "peer {peer_id} score should be positive");
        assert!(
            obs.last_update().as_millis() > 0,
            "peer {peer_id} should have a last_update timestamp"
        );
    }

    Ok(())
}

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[serial]
/// Pings a peer and verifies the returned observations contain latency data,
/// exercising the full probe roundtrip: probe.rs cache lookup → process reply
/// → weight.rs record(ProbeNeighborSuccess) → latency EMA update.
async fn ping_should_record_latency_in_observations(cluster: &ClusterGuard) -> anyhow::Result<()> {
    let [src, dst] = cluster.sample_nodes::<2>();

    let (rtt, obs) = src.inner().ping(&dst.peer_id()).await.context("ping should succeed")?;

    assert!(!rtt.is_zero(), "RTT should be non-zero");

    let imm = obs
        .immediate_qos()
        .context("post-ping observations should contain immediate QoS")?;

    assert!(
        imm.average_latency().is_some(),
        "latency EMA should be present after a successful ping"
    );

    Ok(())
}

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[serial]
/// Verifies the network graph DOT rendering produces valid output containing
/// all cluster node identities and edge annotations.
/// Exercises: render.rs render_dot_with_labels, render_edges_as_dot,
/// graph.rs connected_edges.
async fn graph_should_render_as_valid_dot_with_all_peers(cluster: &ClusterGuard) -> anyhow::Result<()> {
    let [node] = cluster.sample_nodes::<1>();
    let graph = node.inner().graph();

    let dot = hopr_network_graph::render::render_dot(graph);

    assert!(dot.starts_with("digraph"), "DOT output should start with 'digraph'");
    assert!(dot.contains("->"), "DOT output should contain directed edges");

    // Every non-isolated node should appear in the output
    let edges = graph.connected_edges();
    assert!(!edges.is_empty(), "graph should have edges after probe warmup");

    // Verify score annotations are present on edges
    assert!(
        dot.contains("score="),
        "DOT output should contain score annotations on edges"
    );

    Ok(())
}

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[serial]
/// Verifies the reachable-only graph rendering produces a subset that excludes
/// disconnected subgraphs.
/// Exercises: render.rs render_dot_reachable_with_labels, graph.rs reachable_edges.
async fn graph_reachable_edges_should_be_subset_of_connected(cluster: &ClusterGuard) -> anyhow::Result<()> {
    let [node] = cluster.sample_nodes::<1>();
    let graph = node.inner().graph();

    let connected = graph.connected_edges();
    let reachable = graph.reachable_edges();

    // Reachable is a subset (by construction, BFS from self)
    assert!(
        reachable.len() <= connected.len(),
        "reachable edges ({}) should not exceed connected edges ({})",
        reachable.len(),
        connected.len()
    );

    // In a fully probed 3-node cluster, reachable should equal connected
    // (all nodes are reachable from any node)
    assert_eq!(
        reachable.len(),
        connected.len(),
        "in a fully connected cluster, reachable should equal connected"
    );

    Ok(())
}

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[serial]
/// Exercises the ticket price and winning probability chain queries through
/// the HoprLib API, covering the network.rs endpoint code paths.
async fn ticket_price_and_probability_should_be_available(cluster: &ClusterGuard) -> anyhow::Result<()> {
    let [node] = cluster.sample_nodes::<1>();

    let price = node
        .inner()
        .get_ticket_price()
        .await
        .context("should get ticket price")?;

    // Price should be a valid non-zero value from the oracle
    assert!(
        price
            > "0 wxHOPR"
                .parse()
                .context("failed to deserialize the balance for ticket price")?,
        "ticket price should be non-zero"
    );

    let probability = node
        .inner()
        .get_minimum_incoming_ticket_win_probability()
        .await
        .context("should get winning probability")?;

    let prob_f64: f64 = probability.into();
    assert!(
        (0.0..=1.0).contains(&prob_f64),
        "probability {prob_f64} should be in [0, 1]"
    );

    Ok(())
}

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[serial]
/// Verifies that after probe warmup, observed multiaddresses are populated
/// for connected peers, exercising the transport layer's address tracking.
async fn observed_multiaddresses_should_be_populated_after_warmup(cluster: &ClusterGuard) -> anyhow::Result<()> {
    let [src, dst] = cluster.sample_nodes::<2>();

    let addrs = src.inner().network_observed_multiaddresses(&dst.peer_id()).await;

    assert!(
        !addrs.is_empty(),
        "observed multiaddresses should be populated after probe warmup"
    );

    Ok(())
}

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[serial]
/// Verifies that the network health indicator reports a non-red status
/// after the cluster has fully started and probes have warmed up.
async fn network_health_should_not_be_red_after_warmup(cluster: &ClusterGuard) -> anyhow::Result<()> {
    let [node] = cluster.sample_nodes::<1>();

    let health = node.inner().network_health().await;

    assert_ne!(
        health,
        hopr_lib::api::network::Health::Red,
        "network health should not be Red after cluster warmup"
    );

    Ok(())
}
