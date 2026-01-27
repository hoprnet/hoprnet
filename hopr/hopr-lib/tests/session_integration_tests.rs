use std::{
    sync::Arc,
    time::{Duration, SystemTime},
};

use futures::StreamExt;
use hopr_crypto_random::Randomizable;
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_lib::{
    ApplicationDataIn, ApplicationDataOut,
    exports::transport::session::{
        Capabilities, Capability, HoprSession, HoprSessionConfig, SessionId, SessionMetrics, transfer_session,
    },
};
use hopr_network_types::prelude::*;
use hopr_primitive_types::prelude::Address;
use rstest::*;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream, UdpSocket},
    sync::oneshot,
};

#[rstest]
#[case(Capabilities::empty())]
#[case(Capabilities::from(Capability::Segmentation))]
#[tokio::test]
/// Creates paired Hopr sessions bridged to a UDP listener to prove that messages
/// sent over UDP end up in the remote session buffer regardless of capability set.
async fn udp_session_bridging(#[case] cap: Capabilities) -> anyhow::Result<()> {
    let cap_suffix = if cap.is_empty() { "plain" } else { "seg" };
    const BUF_LEN: usize = 16384;
    const MSG_LEN: usize = 9183;

    let start_time = SystemTime::now();

    let dst: Address = (&ChainKeypair::random()).into();
    let id = SessionId::new(1u64, HoprPseudonym::random());
    let (alice_tx, bob_rx) = futures::channel::mpsc::unbounded::<(DestinationRouting, ApplicationDataOut)>();
    let (bob_tx, alice_rx) = futures::channel::mpsc::unbounded::<(DestinationRouting, ApplicationDataOut)>();

    let alice_cfg = HoprSessionConfig {
        capabilities: cap,
        ..Default::default()
    };
    let bob_cfg = HoprSessionConfig {
        capabilities: cap,
        ..Default::default()
    };

    let alice_metrics = Arc::new(SessionMetrics::new(
        id,
        None,
        alice_cfg.frame_mtu,
        alice_cfg.frame_timeout,
        BUF_LEN,
    ));
    let bob_metrics = Arc::new(SessionMetrics::new(
        id,
        None,
        bob_cfg.frame_mtu,
        bob_cfg.frame_timeout,
        BUF_LEN,
    ));

    let mut alice_session = HoprSession::new(
        id,
        DestinationRouting::forward_only(dst, RoutingOptions::Hops(0_u32.try_into()?)),
        alice_cfg,
        (
            alice_tx,
            alice_rx.map(|(_, d)| ApplicationDataIn {
                data: d.data,
                packet_info: Default::default(),
            }),
        ),
        alice_metrics.clone(),
        None,
    )?;

    let mut bob_session = HoprSession::new(
        id,
        DestinationRouting::Return(id.pseudonym().into()),
        bob_cfg,
        (
            bob_tx,
            bob_rx.map(|(_, d)| ApplicationDataIn {
                data: d.data,
                packet_info: Default::default(),
            }),
        ),
        bob_metrics.clone(),
        None,
    )?;

    let mut listener = ConnectedUdpStream::builder()
        .with_buffer_size(BUF_LEN)
        .with_queue_size(512)
        .with_receiver_parallelism(UdpStreamParallelism::Auto)
        .build(("127.0.0.1", 0))?;

    let addr = *listener.bound_address();

    let (ready_tx, ready_rx) = oneshot::channel();
    let transfer_handle = tokio::task::spawn(async move {
        ready_tx.send(()).ok();
        transfer_session(&mut alice_session, &mut listener, BUF_LEN, None).await
    });
    ready_rx.await.ok();

    let msg: [u8; MSG_LEN] = hopr_crypto_random::random_bytes();
    let sender = UdpSocket::bind(("127.0.0.1", 0)).await?;

    let w = sender.send_to(&msg, addr).await?;
    assert_eq!(MSG_LEN, w);

    let mut recv_msg = [0u8; MSG_LEN];
    bob_session.read_exact(&mut recv_msg).await?;

    assert_eq!(recv_msg, msg);

    // Ensure some time passes so uptime > 0 (metrics track ms)
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Verify metrics updated correctly
    let snapshot = alice_metrics.snapshot(0, 0, None);

    // Verify timestamps dynamically
    assert!(
        snapshot.snapshot_at >= start_time,
        "snapshot_at should be recent: {:?}",
        snapshot.snapshot_at
    );
    assert!(
        snapshot.lifetime.created_at >= start_time,
        "created_at should be recent: {:?}, start_time: {:?}",
        snapshot.lifetime.created_at,
        start_time
    );
    assert!(
        snapshot.lifetime.last_activity_at >= start_time,
        "last_activity_at should be recent"
    );
    assert!(snapshot.lifetime.uptime > Duration::ZERO, "uptime should be positive");

    insta::assert_yaml_snapshot!(format!("alice_metrics_udp_{}", cap_suffix), snapshot, {
    ".snapshot_at" => "[snapshot_ts]",
    ".lifetime.created_at" => "[created_at]",
    ".lifetime.last_activity_at" => "[last_activity_at]",
    ".lifetime.uptime" => "[uptime]",
    ".lifetime.idle" => "[idle]",
    ".session_id.pseudonym" => "[pseudonym]",
        });

    let snapshot = bob_metrics.snapshot(0, 0, None);

    // Verify timestamps dynamically
    assert!(snapshot.snapshot_at >= start_time, "snapshot_at should be recent");
    assert!(
        snapshot.lifetime.created_at >= start_time,
        "created_at should be recent"
    );
    assert!(
        snapshot.lifetime.last_activity_at >= start_time,
        "last_activity_at should be recent"
    );
    assert!(snapshot.lifetime.uptime > Duration::ZERO, "uptime should be positive");

    insta::assert_yaml_snapshot!(format!("bob_metrics_udp_{:}", cap_suffix),snapshot, {
        ".snapshot_at" => "[snapshot_ts]",
        ".lifetime.created_at" => "[created_at]",
        ".lifetime.last_activity_at" => "[last_activity_at]",
        ".lifetime.uptime" => "[uptime]",
        ".lifetime.idle" => "[idle]",
        ".session_id.pseudonym" => "[pseudonym]",
    });

    transfer_handle.abort();

    match transfer_handle.await {
        Ok(Err(e)) => panic!("transfer failed: {e}"),
        _ => {} // Task was aborted (expected)
    }

    Ok(())
}

#[rstest]
#[case(Capabilities::empty())]
#[case(Capabilities::from(Capability::Segmentation))]
#[case(Capabilities::from(Capability::RetransmissionAck))]
#[tokio::test]
/// Creates paired Hopr sessions bridged to a TCP listener to prove that messages
/// sent over TCP end up in the remote session buffer regardless of capability set.
async fn tcp_session_bridging(#[case] cap: Capabilities) -> anyhow::Result<()> {
    const BUF_LEN: usize = 16384;
    const MSG_LEN: usize = 9183;

    let start_time = SystemTime::now();

    let dst: Address = (&ChainKeypair::random()).into();
    let id = SessionId::new(1u64, HoprPseudonym::random());
    let (alice_tx, bob_rx) = futures::channel::mpsc::unbounded::<(DestinationRouting, ApplicationDataOut)>();
    let (bob_tx, alice_rx) = futures::channel::mpsc::unbounded::<(DestinationRouting, ApplicationDataOut)>();

    let alice_cfg = HoprSessionConfig {
        capabilities: cap,
        ..Default::default()
    };
    let bob_cfg = HoprSessionConfig {
        capabilities: cap,
        ..Default::default()
    };

    let alice_metrics = Arc::new(SessionMetrics::new(
        id,
        None,
        alice_cfg.frame_mtu,
        alice_cfg.frame_timeout,
        BUF_LEN,
    ));
    let bob_metrics = Arc::new(SessionMetrics::new(
        id,
        None,
        bob_cfg.frame_mtu,
        bob_cfg.frame_timeout,
        BUF_LEN,
    ));

    let mut alice_session = HoprSession::new(
        id,
        DestinationRouting::forward_only(dst, RoutingOptions::Hops(0_u32.try_into()?)),
        alice_cfg,
        (
            alice_tx,
            alice_rx.map(|(_, d)| ApplicationDataIn {
                data: d.data,
                packet_info: Default::default(),
            }),
        ),
        alice_metrics.clone(),
        None,
    )?;

    let mut bob_session = HoprSession::new(
        id,
        DestinationRouting::Return(id.pseudonym().into()),
        bob_cfg,
        (
            bob_tx,
            bob_rx.map(|(_, d)| ApplicationDataIn {
                data: d.data,
                packet_info: Default::default(),
            }),
        ),
        bob_metrics.clone(),
        None,
    )?;

    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;

    let (ready_tx, ready_rx) = oneshot::channel();
    let transfer_handle = tokio::task::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        ready_tx.send(()).ok();
        transfer_session(&mut alice_session, &mut stream, BUF_LEN, None).await
    });

    let msg: [u8; MSG_LEN] = hopr_crypto_random::random_bytes();
    let mut sender = TcpStream::connect(addr).await?;

    ready_rx.await.ok();

    sender.write_all(&msg).await?;

    let mut recv_msg = [0u8; MSG_LEN];
    bob_session.read_exact(&mut recv_msg).await?;

    assert_eq!(recv_msg, msg);

    // Ensure some time passes so uptime > 0 (metrics track ms)
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Verify metrics updated correctly
    let snapshot = alice_metrics.snapshot(0, 0, None);

    // Verify timestamps dynamically
    assert!(
        snapshot.snapshot_at >= start_time,
        "snapshot_at should be recent: {:?}",
        snapshot.snapshot_at
    );
    assert!(
        snapshot.lifetime.created_at >= start_time,
        "created_at should be recent: {:?}, start_time: {:?}",
        snapshot.lifetime.created_at,
        start_time
    );
    assert!(
        snapshot.lifetime.last_activity_at >= start_time,
        "last_activity_at should be recent"
    );
    assert!(snapshot.lifetime.uptime > Duration::ZERO, "uptime should be positive");

    let cap_suffix = if cap.is_empty() {
        "plain"
    } else if cap.contains(Capability::RetransmissionAck) {
        "ack"
    } else {
        "seg"
    };
    insta::assert_yaml_snapshot!(format!("alice_metrics_tcp_{}", cap_suffix), snapshot, {
    ".snapshot_at" => "[snapshot_ts]",
    ".lifetime.created_at" => "[created_at]",
    ".lifetime.last_activity_at" => "[last_activity_at]",
    ".lifetime.uptime" => "[uptime]",
    ".lifetime.idle" => "[idle]",
    ".session_id.pseudonym" => "[pseudonym]",
        });

    let snapshot = bob_metrics.snapshot(0, 0, None);

    // Verify timestamps dynamically
    assert!(snapshot.snapshot_at >= start_time, "snapshot_at should be recent");
    assert!(
        snapshot.lifetime.created_at >= start_time,
        "created_at should be recent"
    );
    assert!(
        snapshot.lifetime.last_activity_at >= start_time,
        "last_activity_at should be recent"
    );
    assert!(snapshot.lifetime.uptime > Duration::ZERO, "uptime should be positive");

    insta::assert_yaml_snapshot!(format!("bob_metrics_tcp_{}", cap_suffix), snapshot, {
        ".snapshot_at" => "[snapshot_ts]",
        ".lifetime.created_at" => "[created_at]",
        ".lifetime.last_activity_at" => "[last_activity_at]",
        ".lifetime.uptime" => "[uptime]",
        ".lifetime.idle" => "[idle]",
        ".session_id.pseudonym" => "[pseudonym]",
    });

    transfer_handle.abort();

    match transfer_handle.await {
        Ok(Err(e)) => panic!("transfer failed: {e}"),
        _ => {} // Task was aborted (expected)
    }

    Ok(())
}
