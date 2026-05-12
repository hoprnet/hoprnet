mod common;

use std::time::Duration;

use common::{
    PEERS_CHAIN, emulate_channel_communication, make_outgoing_packets, peer_setup_for_with_cfg,
    random_packets_of_count, resolve_mock_path,
};
use futures::{SinkExt, StreamExt};
use futures_time::future::FutureExt;
use hopr_api::types::crypto::prelude::*;
use hopr_transport::protocol::{AcknowledgementPipelineConfig, PacketPipelineConfig};
use serial_test::serial;

const TIMEOUT: Duration = Duration::from_secs(30);

/// A very small `ack_out_buffer_size` must not cause a deadlock or panic — some acks may be
/// dropped due to the 50ms sink timeout, but the pipeline continues forwarding packets.
#[serial]
#[test_log::test(tokio::test)]
async fn small_ack_out_buffer_no_deadlock() -> anyhow::Result<()> {
    let packet_count = 10;
    let packets = random_packets_of_count(packet_count);

    let cfg = PacketPipelineConfig {
        ack_config: AcknowledgementPipelineConfig {
            ack_out_buffer_size: 2,
            ..Default::default()
        },
        ..Default::default()
    };

    let (wire_apis, mut apis, _ticket_channels, _processes) = peer_setup_for_with_cfg(3, cfg).await?;

    let path = resolve_mock_path(
        PEERS_CHAIN[0].public().to_address(),
        PEERS_CHAIN[1..3].iter().map(|k| k.public().to_address()).collect(),
    )
    .await?;

    tokio::task::spawn(emulate_channel_communication(wire_apis));

    let out_msgs = make_outgoing_packets(&packets, path);
    apis[0].0.send_all(&mut futures::stream::iter(out_msgs).map(Ok)).await?;

    // At least some packets must arrive at the recipient — the pipeline must not deadlock
    let recv = tokio::time::timeout(Duration::from_secs(15), async {
        let mut received = 0;
        while received < 1 {
            if let Ok(Some(_)) =
                tokio::time::timeout(Duration::from_secs(5), futures::StreamExt::next(&mut apis[2].1)).await
            {
                received += 1;
            } else {
                break;
            }
        }
        received
    })
    .await
    .unwrap_or(0);

    assert!(
        recv >= 1,
        "pipeline must deliver at least one packet even with tiny ack buffer"
    );

    Ok(())
}

/// A very small `ticket_ack_buffer_size` must not cause a deadlock — the pipeline may drop some
/// incoming ack notifications, but forwarding continues.
#[serial]
#[test_log::test(tokio::test)]
async fn small_ticket_ack_buffer_no_deadlock() -> anyhow::Result<()> {
    let packet_count = 10;
    let packets = random_packets_of_count(packet_count);

    let cfg = PacketPipelineConfig {
        ack_config: AcknowledgementPipelineConfig {
            ticket_ack_buffer_size: 2,
            ..Default::default()
        },
        ..Default::default()
    };

    let (wire_apis, mut apis, _ticket_channels, _processes) = peer_setup_for_with_cfg(3, cfg).await?;

    let path = resolve_mock_path(
        PEERS_CHAIN[0].public().to_address(),
        PEERS_CHAIN[1..3].iter().map(|k| k.public().to_address()).collect(),
    )
    .await?;

    tokio::task::spawn(emulate_channel_communication(wire_apis));

    let out_msgs = make_outgoing_packets(&packets, path);
    apis[0].0.send_all(&mut futures::stream::iter(out_msgs).map(Ok)).await?;

    let recv = tokio::time::timeout(Duration::from_secs(15), async {
        let mut received = 0;
        while received < 1 {
            if let Ok(Some(_)) =
                tokio::time::timeout(Duration::from_secs(5), futures::StreamExt::next(&mut apis[2].1)).await
            {
                received += 1;
            } else {
                break;
            }
        }
        received
    })
    .await
    .unwrap_or(0);

    assert!(
        recv >= 1,
        "pipeline must deliver at least one packet even with tiny ticket-ack buffer"
    );

    Ok(())
}

/// `output_concurrency = Some(1)` (serial encoding) must still deliver all packets correctly.
#[serial]
#[test_log::test(tokio::test)]
async fn output_concurrency_one_delivers_all_packets() -> anyhow::Result<()> {
    let packet_count = 5;
    let packets = random_packets_of_count(packet_count);

    let cfg = PacketPipelineConfig {
        output_concurrency: Some(1),
        ..Default::default()
    };

    let (wire_apis, mut apis, mut ticket_channels, _processes) = peer_setup_for_with_cfg(3, cfg).await?;

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
    assert_eq!(
        recv.len(),
        packet_count,
        "all packets must arrive with output_concurrency=1"
    );

    let winning = (&mut ticket_channels[1])
        .take(packet_count)
        .filter(|e| futures::future::ready(e.is_winning_ticket()))
        .count()
        .timeout(futures_time::time::Duration::from(TIMEOUT))
        .await?;
    assert_eq!(winning, packet_count);

    Ok(())
}

/// `input_concurrency = Some(1)` (serial decoding) must still deliver all packets correctly.
#[serial]
#[test_log::test(tokio::test)]
async fn input_concurrency_one_delivers_all_packets() -> anyhow::Result<()> {
    let packet_count = 5;
    let packets = random_packets_of_count(packet_count);

    let cfg = PacketPipelineConfig {
        input_concurrency: Some(1),
        ..Default::default()
    };

    let (wire_apis, mut apis, mut ticket_channels, _processes) = peer_setup_for_with_cfg(3, cfg).await?;

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
    assert_eq!(
        recv.len(),
        packet_count,
        "all packets must arrive with input_concurrency=1"
    );

    let winning = (&mut ticket_channels[1])
        .take(packet_count)
        .filter(|e| futures::future::ready(e.is_winning_ticket()))
        .count()
        .timeout(futures_time::time::Duration::from(TIMEOUT))
        .await?;
    assert_eq!(winning, packet_count);

    Ok(())
}

/// Non-default but valid buffer sizes must still deliver all packets without deadlock.
#[serial]
#[test_log::test(tokio::test)]
async fn custom_buffer_sizes_deliver_all_packets() -> anyhow::Result<()> {
    let packet_count = 8;
    let packets = random_packets_of_count(packet_count);

    let cfg = PacketPipelineConfig {
        ack_config: AcknowledgementPipelineConfig {
            ack_out_buffer_size: 100,
            ticket_ack_buffer_size: 100,
            ack_grouping_capacity: 10,
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

    Ok(())
}

/// Sending a burst of many packets must not deadlock. All must arrive at the recipient.
#[serial]
#[test_log::test(tokio::test)]
async fn large_burst_no_deadlock() -> anyhow::Result<()> {
    let packet_count = 50;
    let packets = random_packets_of_count(packet_count);

    let (wire_apis, mut apis, mut ticket_channels, _processes) = peer_setup_for_with_cfg(3, Default::default()).await?;

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
    assert_eq!(recv.len(), packet_count, "all burst packets must arrive");

    let winning = (&mut ticket_channels[1])
        .take(packet_count)
        .filter(|e| futures::future::ready(e.is_winning_ticket()))
        .count()
        .timeout(futures_time::time::Duration::from(TIMEOUT))
        .await?;
    assert_eq!(winning, packet_count);

    Ok(())
}
