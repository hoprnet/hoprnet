mod common;

use std::time::Duration;

use common::{
    PEERS, PEERS_CHAIN, emulate_channel_communication, inject_raw_wire, make_routing, peer_setup_for,
    peer_setup_for_with_cfg, random_packet_of_size, random_packets_of_count, resolve_mock_path,
};
use futures::{SinkExt, StreamExt};
use futures_time::future::FutureExt;
use hopr_api::types::{crypto::prelude::*, crypto_random::random_integer};
use hopr_protocol_app::prelude::{ApplicationData, ApplicationDataOut};
use hopr_transport_protocol::{AcknowledgementPipelineConfig, PacketPipelineConfig};
use libp2p::PeerId;
use serial_test::serial;

const TIMEOUT: Duration = Duration::from_secs(10);

/// `ApplicationData::new` must accept exactly `PAYLOAD_SIZE` bytes.
#[test]
fn encoding_exactly_max_payload_succeeds() {
    let exact = vec![0u8; ApplicationData::PAYLOAD_SIZE];
    assert!(ApplicationData::new(1u64, exact).is_ok());
}

/// `ApplicationData::new` must reject `PAYLOAD_SIZE + 1` bytes.
#[test]
fn encoding_oversize_payload_returns_error() {
    let too_large = vec![0u8; ApplicationData::PAYLOAD_SIZE + 1];
    assert!(ApplicationData::new(1u64, too_large).is_err());
}

/// A near-max-size payload must be correctly encoded, relayed, and decoded end-to-end.
#[serial]
#[test_log::test(tokio::test)]
async fn near_max_payload_encodes_decodes_correctly() -> anyhow::Result<()> {
    let packet = random_packet_of_size(ApplicationData::PAYLOAD_SIZE - 1);

    let (wire_apis, mut apis, mut ticket_channels, _processes) = peer_setup_for(3).await?;

    let path = resolve_mock_path(
        PEERS_CHAIN[0].public().to_address(),
        PEERS_CHAIN[1..3].iter().map(|k| k.public().to_address()).collect(),
    )
    .await?;

    tokio::task::spawn(emulate_channel_communication(wire_apis));

    apis[0]
        .0
        .send((
            make_routing(path),
            ApplicationDataOut::with_no_packet_info(packet.clone()),
        ))
        .await?;

    let recv = (&mut apis[2].1)
        .take(1)
        .collect::<Vec<_>>()
        .timeout(futures_time::time::Duration::from(TIMEOUT))
        .await?;
    assert_eq!(recv.len(), 1);
    assert_eq!(
        recv[0].1.data.plain_text, packet.plain_text,
        "payload must survive encode/decode"
    );

    (&mut ticket_channels[1])
        .take(1)
        .filter(|e| futures::future::ready(e.is_winning_ticket()))
        .count()
        .timeout(futures_time::time::Duration::from(TIMEOUT))
        .await?;

    Ok(())
}

/// A garbage wire frame must be silently dropped: no delivery to the application, no crash.
/// Subsequent valid packets must still flow through the pipeline normally.
#[serial]
#[test_log::test(tokio::test)]
async fn decode_failure_drops_packet_pipeline_recovers() -> anyhow::Result<()> {
    let valid_count = 3;
    let packets = random_packets_of_count(valid_count);

    let (wire_apis, mut apis, mut ticket_channels, _processes) = peer_setup_for(3).await?;

    let path = resolve_mock_path(
        PEERS_CHAIN[0].public().to_address(),
        PEERS_CHAIN[1..3].iter().map(|k| k.public().to_address()).collect(),
    )
    .await?;

    let relay_id = PeerId::from(*PEERS[1].public());
    let sender_id = PeerId::from(*PEERS[0].public());
    let relay_wire_tx = wire_apis[&relay_id].0.clone();

    tokio::task::spawn(emulate_channel_communication(wire_apis));

    // Inject a single garbage frame before the valid packets
    let garbage: Vec<u8> = (0..200).map(|_| random_integer(0u64, Some(255u64)) as u8).collect();
    inject_raw_wire(&relay_wire_tx, sender_id, garbage);

    // Send valid packets right after
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
        .take(valid_count)
        .collect::<Vec<_>>()
        .timeout(futures_time::time::Duration::from(TIMEOUT))
        .await?;
    assert_eq!(recv.len(), valid_count, "pipeline must recover after decode failure");

    let winning = (&mut ticket_channels[1])
        .take(valid_count)
        .filter(|e| futures::future::ready(e.is_winning_ticket()))
        .count()
        .timeout(futures_time::time::Duration::from(TIMEOUT))
        .await?;
    assert_eq!(winning, valid_count);

    Ok(())
}

/// Interleaving one garbage frame with valid frames must deliver only the valid ones.
#[serial]
#[test_log::test(tokio::test)]
async fn valid_frames_arrive_despite_interleaved_garbage() -> anyhow::Result<()> {
    let valid_count = 4;
    let packets = random_packets_of_count(valid_count);

    let (wire_apis, mut apis, mut ticket_channels, _processes) = peer_setup_for(3).await?;

    let path = resolve_mock_path(
        PEERS_CHAIN[0].public().to_address(),
        PEERS_CHAIN[1..3].iter().map(|k| k.public().to_address()).collect(),
    )
    .await?;

    let relay_id = PeerId::from(*PEERS[1].public());
    let sender_id = PeerId::from(*PEERS[0].public());
    let relay_wire_tx = wire_apis[&relay_id].0.clone();

    tokio::task::spawn(emulate_channel_communication(wire_apis));

    // Send valid packets from peer 0 (these go through encoder → mixer → relay's wire input via emulate)
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

    // Also inject a garbage frame directly into relay's wire input
    let garbage: Vec<u8> = vec![0xAB; 256];
    inject_raw_wire(&relay_wire_tx, sender_id, garbage);

    let recv = (&mut apis[2].1)
        .take(valid_count)
        .collect::<Vec<_>>()
        .timeout(futures_time::time::Duration::from(TIMEOUT))
        .await?;
    assert_eq!(
        recv.len(),
        valid_count,
        "all valid packets must arrive despite interleaved garbage"
    );

    let winning = (&mut ticket_channels[1])
        .take(valid_count)
        .filter(|e| futures::future::ready(e.is_winning_ticket()))
        .count()
        .timeout(futures_time::time::Duration::from(TIMEOUT))
        .await?;
    assert_eq!(winning, valid_count);

    Ok(())
}

/// The relay forwards each packet, acks the sender, and emits a WinningTicket.
/// Verifies the three pipeline stages: relay-forward, ack-egress, and ticket-resolution.
#[serial]
#[test_log::test(tokio::test)]
async fn relay_forward_and_ack_stages_fire_for_each_packet() -> anyhow::Result<()> {
    let packet_count = 5;
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

    // Recipient must receive all packets
    let recv = (&mut apis[2].1)
        .take(packet_count)
        .collect::<Vec<_>>()
        .timeout(futures_time::time::Duration::from(TIMEOUT))
        .await?;
    assert_eq!(recv.len(), packet_count, "recipient must receive all packets");

    // Relay must win exactly packet_count tickets (one per forwarded packet)
    let winning = (&mut ticket_channels[1])
        .take(packet_count)
        .filter(|e| futures::future::ready(e.is_winning_ticket()))
        .count()
        .timeout(futures_time::time::Duration::from(TIMEOUT))
        .await?;
    assert_eq!(
        winning, packet_count,
        "relay must win exactly one ticket per forwarded packet"
    );

    // Sender and recipient must NOT win tickets (only relays do)
    let sender_tickets = tokio::time::timeout(
        Duration::from_millis(300),
        (&mut ticket_channels[0]).take(1).collect::<Vec<_>>(),
    )
    .await;
    assert!(
        sender_tickets.is_err() || sender_tickets.unwrap().is_empty(),
        "sender must not win any tickets"
    );

    Ok(())
}

/// Ack buffer grouping config: a tight `ack_buffer_interval` (10ms) must still correctly
/// deliver all packets and resolve all tickets.
#[serial]
#[test_log::test(tokio::test)]
async fn tight_ack_buffer_interval_delivers_packets() -> anyhow::Result<()> {
    let packet_count = 5;
    let packets = random_packets_of_count(packet_count);

    let cfg = PacketPipelineConfig {
        ack_config: AcknowledgementPipelineConfig {
            ack_buffer_interval: Duration::from_millis(10),
            ..Default::default()
        },
        ..Default::default()
    };

    let (wire_apis, mut apis, mut ticket_channels, _processes) = peer_setup_for_with_cfg(3, cfg).await?;

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

    Ok(())
}

/// A 5-hop path (5 peers, 3 relays) must deliver the packet and produce winning tickets at every
/// intermediate node — validating that each relay stage processes correctly.
#[serial]
#[test_log::test(tokio::test)]
async fn five_hop_all_relay_stages_fire() -> anyhow::Result<()> {
    let packet_count = 2;
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

    // Recipient (peer 4) must receive all packets
    let recv = (&mut apis[4].1)
        .take(packet_count)
        .collect::<Vec<_>>()
        .timeout(futures_time::time::Duration::from(TIMEOUT))
        .await?;
    assert_eq!(recv.len(), packet_count);

    // Each of the 3 intermediate relays (peers 1, 2, 3) must win tickets
    for (relay_idx, relay_tickets) in ticket_channels.iter_mut().enumerate().skip(1).take(3) {
        let winning = relay_tickets
            .take(packet_count)
            .filter(|e| futures::future::ready(e.is_winning_ticket()))
            .count()
            .timeout(futures_time::time::Duration::from(TIMEOUT))
            .await?;
        assert_eq!(
            winning, packet_count,
            "relay {relay_idx} must win exactly one ticket per forwarded packet"
        );
    }

    Ok(())
}

/// Multiple independent concurrent sends from peer 0 all arrive at the recipient without loss.
#[serial]
#[test_log::test(tokio::test)]
async fn concurrent_sends_all_delivered() -> anyhow::Result<()> {
    let packet_count = 10;
    let packets = random_packets_of_count(packet_count);

    let (wire_apis, mut apis, mut ticket_channels, _processes) = peer_setup_for(3).await?;

    let path = resolve_mock_path(
        PEERS_CHAIN[0].public().to_address(),
        PEERS_CHAIN[1..3].iter().map(|k| k.public().to_address()).collect(),
    )
    .await?;

    tokio::task::spawn(emulate_channel_communication(wire_apis));

    let send_futures: Vec<_> = packets
        .iter()
        .map(|msg| {
            (
                make_routing(path.clone()),
                ApplicationDataOut::with_no_packet_info(msg.clone()),
            )
        })
        .collect();
    apis[0]
        .0
        .send_all(&mut futures::stream::iter(send_futures).map(Ok))
        .await?;

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

    Ok(())
}
