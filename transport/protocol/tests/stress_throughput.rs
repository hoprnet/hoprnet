mod common;

use std::time::Duration;

use common::{
    PEERS_CHAIN, emulate_channel_communication, make_routing, peer_setup_for, random_packet_of_size,
    random_packets_of_count, resolve_mock_path,
};
use futures::{SinkExt, StreamExt};
use futures_time::future::FutureExt;
use hopr_api::types::{crypto::prelude::*, crypto_random::random_integer};
use hopr_protocol_app::prelude::{ApplicationData, ApplicationDataOut};
use serial_test::serial;

const LONG_TIMEOUT: Duration = Duration::from_secs(60);

/// 500 packets through a 3-hop path must all arrive — validates pipeline stability under load.
#[serial]
#[test_log::test(tokio::test)]
async fn sustained_500_packets_no_loss() -> anyhow::Result<()> {
    let packet_count = 500;
    let packets = random_packets_of_count(packet_count);

    let (wire_apis, mut apis, mut ticket_channels, _processes) = peer_setup_for(3).await?;

    let path = resolve_mock_path(
        PEERS_CHAIN[0].public().to_address(),
        PEERS_CHAIN[1..3].iter().map(|k| k.public().to_address()).collect(),
    )
    .await?;

    tokio::task::spawn(emulate_channel_communication(wire_apis));

    let out_msgs: Vec<_> = packets
        .iter()
        .map(|msg| {
            (
                make_routing(path.clone()),
                ApplicationDataOut::with_no_packet_info(msg.clone()),
            )
        })
        .collect();
    apis[0].0.send_all(&mut futures::stream::iter(out_msgs).map(Ok)).await?;

    let recv = (&mut apis[2].1)
        .take(packet_count)
        .collect::<Vec<_>>()
        .timeout(futures_time::time::Duration::from(LONG_TIMEOUT))
        .await?;
    assert_eq!(recv.len(), packet_count, "all 500 packets must arrive");

    let winning = (&mut ticket_channels[1])
        .take(packet_count)
        .filter(|e| futures::future::ready(e.is_winning_ticket()))
        .count()
        .timeout(futures_time::time::Duration::from(LONG_TIMEOUT))
        .await?;
    assert_eq!(winning, packet_count, "relay must win 500 tickets");

    Ok(())
}

/// Two sequential bursts separated by an idle window must both fully arrive.
/// Validates that the pipeline does not get stuck in a post-burst stall.
#[serial]
#[test_log::test(tokio::test)]
async fn burst_idle_burst_all_delivered() -> anyhow::Result<()> {
    let burst_size = 100;
    let packets_a = random_packets_of_count(burst_size);
    let packets_b = random_packets_of_count(burst_size);

    let (wire_apis, mut apis, mut ticket_channels, _processes) = peer_setup_for(3).await?;

    let path = resolve_mock_path(
        PEERS_CHAIN[0].public().to_address(),
        PEERS_CHAIN[1..3].iter().map(|k| k.public().to_address()).collect(),
    )
    .await?;

    tokio::task::spawn(emulate_channel_communication(wire_apis));

    // First burst
    let out_a: Vec<_> = packets_a
        .iter()
        .map(|msg| {
            (
                make_routing(path.clone()),
                ApplicationDataOut::with_no_packet_info(msg.clone()),
            )
        })
        .collect();
    apis[0].0.send_all(&mut futures::stream::iter(out_a).map(Ok)).await?;

    let recv_a = (&mut apis[2].1)
        .take(burst_size)
        .collect::<Vec<_>>()
        .timeout(futures_time::time::Duration::from(LONG_TIMEOUT))
        .await?;
    assert_eq!(recv_a.len(), burst_size, "first burst must all arrive");

    (&mut ticket_channels[1])
        .take(burst_size)
        .filter(|e| futures::future::ready(e.is_winning_ticket()))
        .count()
        .timeout(futures_time::time::Duration::from(LONG_TIMEOUT))
        .await?;

    // Idle window
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Second burst
    let out_b: Vec<_> = packets_b
        .iter()
        .map(|msg| {
            (
                make_routing(path.clone()),
                ApplicationDataOut::with_no_packet_info(msg.clone()),
            )
        })
        .collect();
    apis[0].0.send_all(&mut futures::stream::iter(out_b).map(Ok)).await?;

    let recv_b = (&mut apis[2].1)
        .take(burst_size)
        .collect::<Vec<_>>()
        .timeout(futures_time::time::Duration::from(LONG_TIMEOUT))
        .await?;
    assert_eq!(
        recv_b.len(),
        burst_size,
        "second burst must all arrive after idle window"
    );

    (&mut ticket_channels[1])
        .take(burst_size)
        .filter(|e| futures::future::ready(e.is_winning_ticket()))
        .count()
        .timeout(futures_time::time::Duration::from(LONG_TIMEOUT))
        .await?;

    Ok(())
}

/// 200 packets with varying payload sizes (1 byte to max) must all arrive with matching content.
#[serial]
#[test_log::test(tokio::test)]
async fn mixed_payload_sizes_all_delivered() -> anyhow::Result<()> {
    let packet_count = 200;
    let max_size = ApplicationData::PAYLOAD_SIZE;
    let packets: Vec<ApplicationData> = (0..packet_count)
        .map(|_| {
            let size = (random_integer(1u64, Some(max_size as u64 - 1)) as usize).max(1);
            random_packet_of_size(size)
        })
        .collect();

    let (wire_apis, mut apis, mut ticket_channels, _processes) = peer_setup_for(3).await?;

    let path = resolve_mock_path(
        PEERS_CHAIN[0].public().to_address(),
        PEERS_CHAIN[1..3].iter().map(|k| k.public().to_address()).collect(),
    )
    .await?;

    tokio::task::spawn(emulate_channel_communication(wire_apis));

    let out_msgs: Vec<_> = packets
        .iter()
        .map(|msg| {
            (
                make_routing(path.clone()),
                ApplicationDataOut::with_no_packet_info(msg.clone()),
            )
        })
        .collect();
    apis[0].0.send_all(&mut futures::stream::iter(out_msgs).map(Ok)).await?;

    let recv = (&mut apis[2].1)
        .take(packet_count)
        .collect::<Vec<_>>()
        .timeout(futures_time::time::Duration::from(LONG_TIMEOUT))
        .await?;
    assert_eq!(recv.len(), packet_count, "all mixed-payload packets must arrive");

    // Verify all payloads are present (regardless of arrival order)
    let mut sent_sorted: Vec<_> = packets.iter().map(|p| p.plain_text.clone()).collect();
    sent_sorted.sort();
    let mut recv_sorted: Vec<_> = recv.iter().map(|(_, d)| d.data.plain_text.clone()).collect();
    recv_sorted.sort();
    assert_eq!(sent_sorted, recv_sorted, "received payloads must match sent payloads");

    let winning = (&mut ticket_channels[1])
        .take(packet_count)
        .filter(|e| futures::future::ready(e.is_winning_ticket()))
        .count()
        .timeout(futures_time::time::Duration::from(LONG_TIMEOUT))
        .await?;
    assert_eq!(winning, packet_count);

    Ok(())
}

/// 200 packets through a 5-hop path (4 relays) must all arrive and every relay must win tickets.
#[serial]
#[test_log::test(tokio::test)]
async fn five_peer_throughput_all_relays_win() -> anyhow::Result<()> {
    let packet_count = 200;
    let packets = random_packets_of_count(packet_count);

    let (wire_apis, mut apis, mut ticket_channels, _processes) = peer_setup_for(5).await?;

    let path = resolve_mock_path(
        PEERS_CHAIN[0].public().to_address(),
        PEERS_CHAIN[1..5].iter().map(|k| k.public().to_address()).collect(),
    )
    .await?;

    tokio::task::spawn(emulate_channel_communication(wire_apis));

    let out_msgs: Vec<_> = packets
        .iter()
        .map(|msg| {
            (
                make_routing(path.clone()),
                ApplicationDataOut::with_no_packet_info(msg.clone()),
            )
        })
        .collect();
    apis[0].0.send_all(&mut futures::stream::iter(out_msgs).map(Ok)).await?;

    let recv = (&mut apis[4].1)
        .take(packet_count)
        .collect::<Vec<_>>()
        .timeout(futures_time::time::Duration::from(LONG_TIMEOUT))
        .await?;
    assert_eq!(recv.len(), packet_count, "all packets must arrive at peer 4");

    // Each of the 3 intermediate relays (peers 1, 2, 3) must win exactly packet_count tickets
    for (relay_idx, relay_tickets) in ticket_channels.iter_mut().enumerate().skip(1).take(3) {
        let winning = relay_tickets
            .take(packet_count)
            .filter(|e| futures::future::ready(e.is_winning_ticket()))
            .count()
            .timeout(futures_time::time::Duration::from(LONG_TIMEOUT))
            .await?;
        assert_eq!(
            winning, packet_count,
            "relay {relay_idx} must win exactly {packet_count} tickets"
        );
    }

    Ok(())
}
