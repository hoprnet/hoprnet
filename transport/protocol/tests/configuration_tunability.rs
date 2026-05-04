mod common;

use std::time::Duration;

use common::{
    PEERS_CHAIN, emulate_channel_communication, make_routing, peer_setup_for_with_cfg, random_packets_of_count,
    resolve_mock_path,
};
use futures::{SinkExt, StreamExt};
use futures_time::future::FutureExt;
use hopr_api::types::crypto::prelude::*;
use hopr_protocol_app::prelude::ApplicationDataOut;
use hopr_transport_protocol::{AcknowledgementPipelineConfig, PacketPipelineConfig};
use serial_test::serial;
use validator::Validate;

const TIMEOUT: Duration = Duration::from_secs(10);

/// Setting `ack_buffer_interval` to the minimum (10ms) must still correctly deliver all packets
/// and resolve all tickets.
#[serial]
#[test_log::test(tokio::test)]
async fn ack_buffer_interval_minimum_still_delivers_packets() -> anyhow::Result<()> {
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

/// `output_concurrency = Some(0)` falls back to the default concurrency and packets still flow.
#[serial]
#[test_log::test(tokio::test)]
async fn output_concurrency_zero_falls_back_to_default() -> anyhow::Result<()> {
    let packet_count = 3;
    let packets = random_packets_of_count(packet_count);

    let cfg = PacketPipelineConfig {
        output_concurrency: Some(0),
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

/// `ack_grouping_capacity = 0` must fail validation with an error naming the field.
#[test]
fn ack_grouping_capacity_zero_validation_fails() {
    let cfg = AcknowledgementPipelineConfig {
        ack_grouping_capacity: 0,
        ..Default::default()
    };
    let result = cfg.validate();
    assert!(result.is_err(), "ack_grouping_capacity=0 must not validate");
    let errors = result.unwrap_err();
    assert!(
        errors.field_errors().contains_key("ack_grouping_capacity"),
        "validation error must reference the ack_grouping_capacity field"
    );
}

/// `ack_buffer_interval` below 10ms must fail validation.
#[test]
fn ack_buffer_interval_too_short_validation_fails() {
    let cfg = AcknowledgementPipelineConfig {
        ack_buffer_interval: Duration::from_millis(5),
        ..Default::default()
    };
    let result = cfg.validate();
    assert!(result.is_err(), "ack_buffer_interval < 10ms must not validate");
    let errors = result.unwrap_err();
    assert!(
        errors.field_errors().contains_key("ack_buffer_interval"),
        "validation error must reference the ack_buffer_interval field"
    );
}
