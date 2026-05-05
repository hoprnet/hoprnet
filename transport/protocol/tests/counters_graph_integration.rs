mod common;

use std::time::Duration;

use common::{
    PEERS, PEERS_CHAIN, emulate_channel_communication, make_outgoing_packets, peer_setup_for_with_counters,
    random_packets_of_count, resolve_mock_path,
};
use futures::{SinkExt, StreamExt};
use futures_time::future::FutureExt;
use hopr_api::types::crypto::prelude::*;
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
    let (_, msgs_sent, _) = entry.unwrap();
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
