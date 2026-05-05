mod common;

use std::time::Duration;

use common::{
    PEERS, PEERS_CHAIN, emulate_channel_communication, inject_raw_wire, make_outgoing_packets, make_routing,
    peer_setup_for, random_packet_of_size, random_packets_of_count, resolve_mock_path,
};
use futures::{SinkExt, StreamExt};
use futures_time::future::FutureExt;
use hopr_api::types::crypto::prelude::*;
use hopr_protocol_app::prelude::{ApplicationData, ApplicationDataOut};
use libp2p::PeerId;
use serial_test::serial;

const TIMEOUT: Duration = Duration::from_secs(10);

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

    let winning = (&mut ticket_channels[1])
        .take(1)
        .filter(|e| futures::future::ready(e.is_winning_ticket()))
        .count()
        .timeout(futures_time::time::Duration::from(TIMEOUT))
        .await?;
    assert_eq!(winning, 1, "relay must win exactly one ticket for the forwarded packet");

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
    let out_msgs = make_outgoing_packets(&packets, path.clone());
    apis[0].0.send_all(&mut futures::stream::iter(out_msgs).map(Ok)).await?;

    // Also inject a garbage frame directly into relay's wire input
    inject_raw_wire(&relay_wire_tx, sender_id, vec![0xAB; 256])?;

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

    let out_msgs = make_outgoing_packets(&packets, path);
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

    let recipient_tickets = tokio::time::timeout(
        Duration::from_millis(300),
        (&mut ticket_channels[2]).take(1).collect::<Vec<_>>(),
    )
    .await;
    assert!(
        recipient_tickets.is_err() || recipient_tickets.unwrap().is_empty(),
        "recipient must not win any tickets"
    );

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

    let out_msgs = make_outgoing_packets(&packets, path);
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

    let sender = apis[0].0.clone();
    futures::future::try_join_all(packets.into_iter().map(|msg| {
        let mut sender = sender.clone();
        let routing = make_routing(path.clone());
        async move {
            sender
                .send((routing, ApplicationDataOut::with_no_packet_info(msg)))
                .await
        }
    }))
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
