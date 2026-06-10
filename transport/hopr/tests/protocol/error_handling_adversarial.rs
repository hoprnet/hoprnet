mod common;

use std::time::Duration;

use common::{
    PEERS, PEERS_CHAIN, corrupt_bytes, emulate_channel_communication, inject_raw_wire, make_outgoing_packets,
    make_routing, peer_setup_for, random_packets_of_count, resolve_mock_path,
};
use futures::{SinkExt, StreamExt};
use futures_time::future::FutureExt;
use hopr_api::types::{crypto::prelude::*, crypto_random::random_integer};
use hopr_crypto_packet::prelude::HoprPacket;
use hopr_protocol_app::prelude::{ApplicationData, ApplicationDataOut};
use libp2p::PeerId;
use serial_test::serial;

const TIMEOUT: Duration = Duration::from_secs(10);
const SHORT_WAIT: Duration = Duration::from_millis(600);

/// Random bytes injected into the wire must be silently dropped — no delivery to the application,
/// no panic, no WinningTicket at the relay.
#[serial]
#[test_log::test(tokio::test)]
async fn random_bytes_dropped_no_delivery() -> anyhow::Result<()> {
    let (wire_apis, mut apis, mut ticket_channels, _processes) = peer_setup_for(3).await?;

    let relay_id = PeerId::from(*PEERS[1].public());
    let sender_id = PeerId::from(*PEERS[0].public());
    let relay_wire_tx = wire_apis[&relay_id].0.clone();

    tokio::task::spawn(emulate_channel_communication(wire_apis));

    // Inject 20 random-sized, random-content byte blobs into the relay's wire input
    for _ in 0..20 {
        let len = random_integer(32u64, Some(512u64)) as usize;
        let garbage: Vec<u8> = (0..len).map(|_| random_integer(0u64, Some(255u64)) as u8).collect();
        inject_raw_wire(&relay_wire_tx, sender_id, garbage)?;
    }

    // Wait longer than PACKET_DECODING_TIMEOUT (150ms) + ack_buffer_interval (default 50ms)
    tokio::time::sleep(SHORT_WAIT).await;

    // Nothing should have been delivered to the recipient
    let received = tokio::time::timeout(Duration::from_millis(100), apis[2].1.next()).await;
    assert!(received.is_err(), "no delivery expected for garbage input");

    // No winning ticket at relay either
    let ticket_result = tokio::time::timeout(Duration::from_millis(100), ticket_channels[1].next()).await;
    assert!(ticket_result.is_err(), "no ticket event expected for garbage input");

    Ok(())
}

/// Packets shorter than a valid HOPR packet must be silently dropped with no crash.
#[serial]
#[test_log::test(tokio::test)]
async fn truncated_packets_dropped_no_crash() -> anyhow::Result<()> {
    let (wire_apis, mut apis, mut ticket_channels, _processes) = peer_setup_for(3).await?;

    let relay_id = PeerId::from(*PEERS[1].public());
    let sender_id = PeerId::from(*PEERS[0].public());
    let relay_wire_tx = wire_apis[&relay_id].0.clone();

    tokio::task::spawn(emulate_channel_communication(wire_apis));

    // Inject too-short packets: 1 byte, 16 bytes, 100 bytes
    for len in [1usize, 16, 100] {
        let short: Vec<u8> = vec![0xAB; len];
        inject_raw_wire(&relay_wire_tx, sender_id, short)?;
    }

    tokio::time::sleep(SHORT_WAIT).await;

    let received = tokio::time::timeout(Duration::from_millis(100), apis[2].1.next()).await;
    assert!(received.is_err(), "no delivery expected for truncated input");

    let ticket_result = tokio::time::timeout(Duration::from_millis(100), ticket_channels[1].next()).await;
    assert!(ticket_result.is_err(), "no ticket event expected for truncated input");

    Ok(())
}

/// `ApplicationData::new` must return an error when the payload exceeds `PAYLOAD_SIZE`.
#[test]
fn oversize_payload_rejected_at_construction() {
    let too_large = vec![0u8; ApplicationData::PAYLOAD_SIZE + 1];
    assert!(
        ApplicationData::new(42u64, too_large).is_err(),
        "payload larger than PAYLOAD_SIZE must be rejected"
    );
}

/// `ApplicationData::new` must succeed for exactly `PAYLOAD_SIZE` bytes.
#[test]
fn exact_max_payload_accepted() {
    let exact = vec![0u8; ApplicationData::PAYLOAD_SIZE];
    assert!(
        ApplicationData::new(42u64, exact).is_ok(),
        "payload of exactly PAYLOAD_SIZE must be accepted"
    );
}

/// After injecting garbage, subsequent valid packets still flow end-to-end.
/// This validates that undecodable input does not permanently disrupt the pipeline.
#[serial]
#[test_log::test(tokio::test)]
async fn pipeline_continues_after_garbage_input() -> anyhow::Result<()> {
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

    // First inject garbage into the relay
    for _ in 0..30 {
        let len = random_integer(64u64, Some(256u64)) as usize;
        let garbage: Vec<u8> = (0..len).map(|_| random_integer(0u64, Some(255u64)) as u8).collect();
        inject_raw_wire(&relay_wire_tx, sender_id, garbage)?;
    }

    // Then send valid packets through the normal path
    let out_msgs = make_outgoing_packets(&packets, path);
    apis[0].0.send_all(&mut futures::stream::iter(out_msgs).map(Ok)).await?;

    // All valid packets must arrive
    let recv = (&mut apis[2].1)
        .take(valid_count)
        .collect::<Vec<_>>()
        .timeout(futures_time::time::Duration::from(TIMEOUT))
        .await?;
    assert_eq!(
        recv.len(),
        valid_count,
        "all valid packets must arrive after garbage was injected"
    );

    // All relay tickets must be won
    let winning = (&mut ticket_channels[1])
        .take(valid_count)
        .filter(|e| futures::future::ready(e.is_winning_ticket()))
        .count()
        .timeout(futures_time::time::Duration::from(TIMEOUT))
        .await?;
    assert_eq!(winning, valid_count);

    Ok(())
}

/// Mix of garbage and valid traffic: the valid packets all arrive at the destination.
#[serial]
#[test_log::test(tokio::test)]
async fn flooding_garbage_does_not_starve_valid_traffic() -> anyhow::Result<()> {
    let valid_count = 5;
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

    // Concurrently inject 50 garbage blobs AND send valid traffic
    let garbage_task = {
        let relay_wire_tx = relay_wire_tx.clone();
        tokio::task::spawn(async move {
            for _ in 0..50 {
                let len = random_integer(32u64, Some(500u64)) as usize;
                let garbage: Vec<u8> = (0..len).map(|_| random_integer(0u64, Some(255u64)) as u8).collect();
                inject_raw_wire(&relay_wire_tx, sender_id, garbage)?;
            }
            anyhow::Ok(())
        })
    };

    let out_msgs = make_outgoing_packets(&packets, path);
    apis[0].0.send_all(&mut futures::stream::iter(out_msgs).map(Ok)).await?;
    garbage_task.await??;

    let recv = (&mut apis[2].1)
        .take(valid_count)
        .collect::<Vec<_>>()
        .timeout(futures_time::time::Duration::from(TIMEOUT))
        .await?;
    assert_eq!(
        recv.len(),
        valid_count,
        "valid packets must arrive despite concurrent garbage flooding"
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

/// A corrupt wire frame (right length, flipped bytes) must not crash the pipeline or deliver to app.
#[serial]
#[test_log::test(tokio::test)]
async fn corrupt_wire_frame_does_not_crash_pipeline() -> anyhow::Result<()> {
    let valid_count = 2;
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

    // Inject a frame of the exact HOPR wire-packet size but with corrupted content
    let frame = vec![0u8; HoprPacket::SIZE];
    let corrupted = corrupt_bytes(&frame, 42);
    inject_raw_wire(&relay_wire_tx, sender_id, corrupted)?;

    // Also inject an all-zeros frame of valid size
    inject_raw_wire(&relay_wire_tx, sender_id, vec![0u8; HoprPacket::SIZE])?;

    // Valid packets must still flow after the pipeline processes the corrupt frames
    let out_msgs = make_outgoing_packets(&packets, path);
    apis[0].0.send_all(&mut futures::stream::iter(out_msgs).map(Ok)).await?;

    let recv = (&mut apis[2].1)
        .take(valid_count)
        .collect::<Vec<_>>()
        .timeout(futures_time::time::Duration::from(TIMEOUT))
        .await?;
    assert_eq!(
        recv.len(),
        valid_count,
        "valid packets must arrive after pipeline handled corrupt frames"
    );

    let winning = (&mut ticket_channels[1])
        .take(valid_count)
        .filter(|e| futures::future::ready(e.is_winning_ticket()))
        .count()
        .timeout(futures_time::time::Duration::from(TIMEOUT))
        .await?;
    assert_eq!(
        winning, valid_count,
        "all relay ticket events should be winning after corrupt-frame recovery"
    );

    Ok(())
}

/// Sending the same packet content twice must not crash — replay detection handles duplicates gracefully.
#[serial]
#[test_log::test(tokio::test)]
async fn duplicate_send_does_not_crash_pipeline() -> anyhow::Result<()> {
    // Each send re-encodes into a fresh Sphinx packet, so replay detection at the wire level is not
    // triggered. Both sends may or may not be delivered, but neither must panic.
    let payload = b"duplicate payload for replay test";
    let packet = ApplicationData::new(99u64, payload.as_ref())?;

    let (wire_apis, mut apis, mut ticket_channels, _processes) = peer_setup_for(3).await?;

    let path = resolve_mock_path(
        PEERS_CHAIN[0].public().to_address(),
        PEERS_CHAIN[1..3].iter().map(|k| k.public().to_address()).collect(),
    )
    .await?;

    tokio::task::spawn(emulate_channel_communication(wire_apis));

    // Send same payload content twice through separate outgoing calls
    for _ in 0..2 {
        apis[0]
            .0
            .send((
                make_routing(path.clone()),
                ApplicationDataOut::with_no_packet_info(packet.clone()),
            ))
            .await?;
    }

    // At least the first delivery should arrive (the second may be filtered by replay detection)
    let recv = (&mut apis[2].1)
        .take(1)
        .collect::<Vec<_>>()
        .timeout(futures_time::time::Duration::from(TIMEOUT))
        .await?;
    assert!(!recv.is_empty(), "at least one of the duplicate packets must arrive");

    // Consume any ticket events to keep the pipeline clean
    let _ = tokio::time::timeout(
        Duration::from_millis(500),
        (&mut ticket_channels[1]).take(2).collect::<Vec<_>>(),
    )
    .await;

    Ok(())
}
