mod common;

use std::time::Duration;

use anyhow::Context;
use common::{
    PEERS, PEERS_CHAIN, emulate_channel_communication, make_outgoing_packets, peer_setup_for_with_counters,
    random_packets_of_count, resolve_mock_path,
};
use futures::{SinkExt, StreamExt};
use futures_time::future::FutureExt;
use hopr_api::{
    graph::{
        NetworkGraphView, NetworkGraphWrite,
        traits::{EdgeImmediateProtocolObservable, EdgeObservableRead, EdgeObservableWrite, EdgeWeightType},
    },
    types::{
        crypto::prelude::*,
        crypto_random::Randomizable,
        internal::{errors::PathError, prelude::VerifiedAcknowledgement},
    },
};
use hopr_network_graph::ChannelGraph;
use hopr_transport::{
    ConformanceReport, PeerProtocolCounterRegistry,
    path::{HoprGraphPathSelector, PathPlannerConfig, PathPlannerError, traits::PathSelector},
};
use serial_test::serial;

const TIMEOUT: Duration = Duration::from_secs(10);

/// After sending N packets, sender's counter registry must record N messages sent to the relay.
#[serial]
#[test_log::test(tokio::test)]
async fn counters_drain_matches_packet_count() -> anyhow::Result<()> {
    let packet_count = 3;
    let packets = random_packets_of_count(packet_count);

    let (wire_apis, mut apis, mut ticket_channels, _processes, counters) = peer_setup_for_with_counters(3).await?;

    let path = resolve_mock_path(
        PEERS_CHAIN[0].public().to_address(),
        PEERS_CHAIN[1..3].iter().map(|k| k.public().to_address()).collect(),
    )
    .await?;

    tokio::task::spawn(emulate_channel_communication(wire_apis));

    let out_msgs = make_outgoing_packets(&packets, path);
    apis[0].0.send_all(&mut futures::stream::iter(out_msgs).map(Ok)).await?;

    let recv = (&mut apis[2].1)
        .take(packet_count)
        .collect::<Vec<_>>()
        .timeout(futures_time::time::Duration::from(TIMEOUT))
        .await?;
    assert_eq!(recv.len(), packet_count);

    let winning = (&mut ticket_channels[1])
        .take(packet_count)
        .filter(|e| futures::future::ready(e.is_winning_ticket()))
        .count()
        .timeout(futures_time::time::Duration::from(TIMEOUT))
        .await?;
    assert_eq!(winning, packet_count);

    let sender_drain = counters[0].drain();
    assert!(
        !sender_drain.is_empty(),
        "sender should have non-zero counters after traffic"
    );

    let relay_key = *PEERS[1].public();
    let entry = sender_drain.iter().find(|(k, ..)| *k == relay_key);
    assert!(entry.is_some(), "sender counter must track messages sent to the relay");
    let (_, msgs_sent, _) = entry.expect("sender counter must track messages sent to the relay");
    assert_eq!(*msgs_sent, packet_count as u64, "sent count must match packet_count");

    Ok(())
}

/// A fresh registry with no traffic must drain to an empty result.
#[serial]
#[test_log::test(tokio::test)]
async fn counter_registry_zero_for_idle_peers() -> anyhow::Result<()> {
    let (_wire_apis, _apis, _ticket_channels, _processes, counters) = peer_setup_for_with_counters(3).await?;

    for (i, registry) in counters.iter().enumerate() {
        let drain = registry.drain();
        assert!(
            drain.is_empty(),
            "peer {i} should have zero counters before any traffic"
        );
    }

    Ok(())
}

/// After the first drain, a second drain must return empty — confirming counters are reset.
#[serial]
#[test_log::test(tokio::test)]
async fn counter_drain_resets_state() -> anyhow::Result<()> {
    let packet_count = 2;
    let packets = random_packets_of_count(packet_count);

    let (wire_apis, mut apis, mut ticket_channels, _processes, counters) = peer_setup_for_with_counters(3).await?;

    let path = resolve_mock_path(
        PEERS_CHAIN[0].public().to_address(),
        PEERS_CHAIN[1..3].iter().map(|k| k.public().to_address()).collect(),
    )
    .await?;

    tokio::task::spawn(emulate_channel_communication(wire_apis));

    let out_msgs = make_outgoing_packets(&packets, path);
    apis[0].0.send_all(&mut futures::stream::iter(out_msgs).map(Ok)).await?;

    (&mut apis[2].1)
        .take(packet_count)
        .collect::<Vec<_>>()
        .timeout(futures_time::time::Duration::from(TIMEOUT))
        .await?;
    (&mut ticket_channels[1])
        .take(packet_count)
        .filter(|e| futures::future::ready(e.is_winning_ticket()))
        .count()
        .timeout(futures_time::time::Duration::from(TIMEOUT))
        .await?;

    let first_drain = counters[0].drain();
    assert!(!first_drain.is_empty(), "first drain should be non-empty after traffic");

    let second_drain = counters[0].drain();
    assert!(
        second_drain.is_empty(),
        "second drain should be empty after counters reset"
    );

    Ok(())
}

/// Mark an edge fully usable for routing: connected, probed, funded.
fn mark_edge_full(graph: &ChannelGraph, src: &OffchainPublicKey, dst: &OffchainPublicKey) {
    graph.upsert_edge(src, dst, |obs| {
        obs.record(EdgeWeightType::Connected(true));
        obs.record(EdgeWeightType::Immediate(Ok(Duration::from_millis(50))));
        obs.record(EdgeWeightType::Intermediate(Ok(Duration::from_millis(50))));
        obs.record(EdgeWeightType::Balance(Some(hopr_api::graph::traits::Balance::from(
            1000u64,
        ))));
    });
}

/// A bidirectional 1-hop cluster `me <-> relay <-> dest`, plus a selector that reads it. The returned
/// graph shares state with the selector's, so conformance recorded on it is what the selector sees.
fn one_hop_cluster(
    me: OffchainPublicKey,
    relay: OffchainPublicKey,
    dest: OffchainPublicKey,
) -> (ChannelGraph, HoprGraphPathSelector<ChannelGraph>) {
    let graph = ChannelGraph::new(me);
    graph.add_node(relay);
    graph.add_node(dest);
    for (src, dst) in [(me, relay), (relay, dest), (dest, relay), (relay, me)] {
        graph.add_edge(&src, &dst).expect("edge should be addable");
        mark_edge_full(&graph, &src, &dst);
    }
    let cfg = PathPlannerConfig::default();
    let selector = HoprGraphPathSelector::new(
        me,
        graph.clone(),
        4,
        cfg.edge_penalty,
        cfg.min_ack_rate,
        cfg.min_loopback_rate,
        cfg.min_paths_anonymity_floor,
    );
    (graph, selector)
}

/// Record a drained conformance report onto the `src -> peer` edges, exactly as the counter-flush
/// task in `lib.rs` does.
fn apply_conformance_report(graph: &ChannelGraph, src: &OffchainPublicKey, report: &[ConformanceReport]) {
    for report in report {
        graph.upsert_edge(src, &report.peer, |obs| {
            obs.record(EdgeWeightType::ImmediateProtocolConformance {
                num_packets: report.sent,
                num_acks: report.acks,
            });
            if report.relayed > 0 || report.genuine_acks > 0 {
                obs.record(EdgeWeightType::RelayedProtocolConformance {
                    num_relayed: report.relayed,
                    num_genuine_acks: report.genuine_acks,
                });
            }
        });
    }
}

/// A cold burst of 10 packets to the relay whose Proof-of-Relay acknowledgments arrive only after the
/// flush began, drained with `grace`. A zero grace reproduces the flush as it shipped, reading both
/// counters at the same instant.
///
/// The ordering is driven by polls, not by racing two timers, as in the `protocol::counters` unit
/// tests: a stalled executor could otherwise poll the drain past its grace before the acks are
/// recorded, and the graced report would read none.
async fn cold_burst_report(relay: &OffchainPublicKey, grace: Duration) -> anyhow::Result<Vec<ConformanceReport>> {
    let registry = PeerProtocolCounterRegistry::default();
    let counters = registry.get_or_create(relay);
    for _ in 0..10 {
        counters.record_message_sent();
    }

    let mut drain = std::pin::pin!(registry.drain_settled(grace));
    if grace.is_zero() {
        // Nothing to wait for: the report is complete before the acks arrive.
        let report = drain.await;
        counters.record_acks_received(10);
        return Ok(report);
    }

    // The first poll takes the sent snapshot and starts the grace, so the acks land inside it.
    anyhow::ensure!(
        futures::poll!(drain.as_mut()).is_pending(),
        "the drain must be waiting out its grace before the acks are recorded"
    );
    counters.record_acks_received(10);
    Ok(drain.await)
}

/// Reproduces hoprnet#8484 across counter report, graph edge weight, and path selection.
///
/// On a cold burst the Proof-of-Relay acknowledgments lag the sends. The counter flush as it shipped
/// read sent packets and received acks at the same instant, so a burst whose acks are still in flight
/// is reported as (sent=10, acks=0). Recorded onto the relay edge that drops its ack rate below
/// `min_ack_rate`, and the selector can no longer find the 1-hop route — the exact "cannot find 1 hop
/// path" outage. Waiting an acknowledgment grace credits those acks to the same report, so the relay
/// stays eligible and the route holds. The grace is the thing that decides it: the same burst flushed
/// with no grace still loses the route.
#[serial]
#[test_log::test(tokio::test)]
async fn a_cold_burst_flush_decides_whether_the_one_hop_route_survives() -> anyhow::Result<()> {
    let me = *PEERS[0].public();
    let relay = *PEERS[1].public();
    let dest = *PEERS[2].public();

    // The instant flush (no grace) reproduces the bug: the in-flight acks miss the report and the
    // 1-hop route disappears.
    let (graph, selector) = one_hop_cluster(me, relay, dest);
    assert!(
        selector.select_path(me, dest, 1).is_ok(),
        "precondition: the 1-hop route exists before the flush"
    );
    apply_conformance_report(&graph, &me, &cold_burst_report(&relay, Duration::ZERO).await?);
    assert!(
        matches!(
            selector.select_path(me, dest, 1),
            Err(PathPlannerError::Path(PathError::PathNotFound(..)))
        ),
        "the instant flush dropped the relay below min_ack_rate and lost the 1-hop route"
    );

    // The graced flush credits the in-flight acks, so the same burst keeps the route.
    let (graph, selector) = one_hop_cluster(me, relay, dest);
    apply_conformance_report(
        &graph,
        &me,
        &cold_burst_report(&relay, Duration::from_millis(200)).await?,
    );
    assert!(
        selector.select_path(me, dest, 1).is_ok(),
        "the grace keeps the relay above min_ack_rate, so the 1-hop route survives"
    );

    Ok(())
}

/// 0-hop packets must not count as relayed; 1-hop packets and their genuine acks must.
#[serial]
#[test_log::test(tokio::test)]
async fn only_relayed_packets_and_their_genuine_acks_feed_the_genuine_ack_counters() -> anyhow::Result<()> {
    let packet_count = 3;
    let (wire_apis, mut apis, _ticket_channels, _processes, counters) = peer_setup_for_with_counters(3).await?;
    tokio::task::spawn(emulate_channel_communication(wire_apis));

    let zero_hop = resolve_mock_path(
        PEERS_CHAIN[0].public().to_address(),
        vec![PEERS_CHAIN[1].public().to_address()],
    )
    .await?;
    let out_msgs = make_outgoing_packets(&random_packets_of_count(packet_count), zero_hop);
    apis[0].0.send_all(&mut futures::stream::iter(out_msgs).map(Ok)).await?;
    (&mut apis[1].1)
        .take(packet_count)
        .collect::<Vec<_>>()
        .timeout(futures_time::time::Duration::from(TIMEOUT))
        .await?;

    let one_hop = resolve_mock_path(
        PEERS_CHAIN[0].public().to_address(),
        PEERS_CHAIN[1..3].iter().map(|k| k.public().to_address()).collect(),
    )
    .await?;
    let out_msgs = make_outgoing_packets(&random_packets_of_count(packet_count), one_hop);
    apis[0].0.send_all(&mut futures::stream::iter(out_msgs).map(Ok)).await?;
    (&mut apis[2].1)
        .take(packet_count)
        .collect::<Vec<_>>()
        .timeout(futures_time::time::Duration::from(TIMEOUT))
        .await?;

    let relay = *PEERS[1].public();
    let report = counters[0].drain_settled(Duration::from_secs(1)).await;
    let report = report
        .iter()
        .find(|report| report.peer == relay)
        .context("the relay must be reported")?;
    assert_eq!(
        report.sent,
        2 * packet_count as u64,
        "every packet still counts as sent"
    );
    assert_eq!(
        report.relayed, packet_count as u64,
        "only the 1-hop packets count as relayed"
    );
    assert_eq!(
        report.genuine_acks, packet_count as u64,
        "only their acks count as genuine"
    );

    Ok(())
}

/// Report for `packet_count` relayed packets, acked genuinely or with random acks.
fn relay_ack_report(relay_key: &OffchainKeypair, packet_count: usize, relay_rejects: bool) -> Vec<ConformanceReport> {
    let registry = PeerProtocolCounterRegistry::default();
    let counters = registry.get_or_create(relay_key.public());

    let mut acks = Vec::new();
    for _ in 0..packet_count {
        let half_key = HalfKey::random();
        let challenge = half_key.to_challenge().expect("random half key has a challenge");
        counters.record_message_sent();
        counters.record_relayed_message_sent(challenge);
        let ack = if relay_rejects {
            VerifiedAcknowledgement::random(relay_key)
        } else {
            VerifiedAcknowledgement::new(half_key, relay_key)
        };
        acks.push(ack.leak());
    }
    counters.record_acks_received(acks.len() as u64);
    counters.record_genuine_acks(relay_key.public(), &acks);

    futures::executor::block_on(registry.drain_settled(Duration::ZERO))
}

fn best_path_cost(
    selector: &HoprGraphPathSelector<ChannelGraph>,
    me: OffchainPublicKey,
    dest: OffchainPublicKey,
) -> f64 {
    selector
        .select_path(me, dest, 1)
        .map(|paths| paths.iter().map(|path| path.cost).fold(0.0, f64::max))
        .unwrap_or(0.0)
}

/// A relay answering with random acks keeps its ack rate but is starved by its genuine-ack rate.
#[serial]
#[test_log::test(tokio::test)]
async fn a_relay_answering_with_random_acks_is_starved_despite_its_ack_rate() -> anyhow::Result<()> {
    let me = *PEERS[0].public();
    let relay = *PEERS[1].public();
    let dest = *PEERS[2].public();

    let (rejecting_graph, rejecting_selector) = one_hop_cluster(me, relay, dest);
    apply_conformance_report(&rejecting_graph, &me, &relay_ack_report(&PEERS[1], 20, true));
    let ack_rate = rejecting_graph
        .edge(&me, &relay)
        .and_then(|obs| obs.immediate_qos().and_then(|imm| imm.ack_rate()))
        .context("the relay edge must have an ack rate")?;
    assert!(
        ack_rate >= PathPlannerConfig::default().min_ack_rate,
        "random acks keep the plain ack rate up, got {ack_rate}"
    );

    let (honest_graph, honest_selector) = one_hop_cluster(me, relay, dest);
    apply_conformance_report(&honest_graph, &me, &relay_ack_report(&PEERS[1], 20, false));

    let rejecting = best_path_cost(&rejecting_selector, me, dest);
    let honest = best_path_cost(&honest_selector, me, dest);
    assert!(honest > 0.0, "an honest relay stays a first hop");
    assert!(
        rejecting < honest * 1e-6,
        "a rejecting relay must be starved: {rejecting} against {honest}"
    );

    Ok(())
}
