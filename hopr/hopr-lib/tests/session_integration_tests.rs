use std::{sync::Arc, time::SystemTime};

use futures::StreamExt;
use hopr_crypto_random::Randomizable;
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_lib::{
    ApplicationDataIn, ApplicationDataOut,
    exports::transport::session::{
        AtomicSurbFlowEstimator, BalancerStateData, Capabilities, Capability, HoprSession, HoprSessionConfig,
        SessionId, SessionTelemetry, transfer_session,
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

    let alice_metrics = Arc::new(SessionTelemetry::new(id, alice_cfg));

    let bob_metrics = Arc::new(SessionTelemetry::new(id, bob_cfg));

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
        None,
        alice_metrics.clone(),
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
        None,
        bob_metrics.clone(),
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

    // Verify metrics updated correctly
    let snapshot = alice_metrics.snapshot();

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

    insta::assert_yaml_snapshot!(format!("alice_metrics_udp_{}", cap_suffix), snapshot, {
    ".snapshot_at" => "[snapshot_ts]",
    ".lifetime.created_at" => "[created_at]",
    ".lifetime.last_activity_at" => "[last_activity_at]",
    ".lifetime.uptime" => "[uptime]",
    ".lifetime.idle" => "[idle]",
    ".session_id.pseudonym" => "[pseudonym]",
        });

    let snapshot = bob_metrics.snapshot();

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

    insta::assert_yaml_snapshot!(format!("bob_metrics_udp_{:}", cap_suffix),snapshot, {
        ".snapshot_at" => "[snapshot_ts]",
        ".lifetime.created_at" => "[created_at]",
        ".lifetime.last_activity_at" => "[last_activity_at]",
        ".lifetime.uptime" => "[uptime]",
        ".lifetime.idle" => "[idle]",
        ".session_id.pseudonym" => "[pseudonym]",
    });

    transfer_handle.abort();

    Ok(())
}

#[rstest]
#[case(Capabilities::empty())]
#[case(Capabilities::from(Capability::Segmentation))]
#[case(Capabilities::from(Capability::RetransmissionAck))]
#[case(Capabilities::from(Capability::RetransmissionNack))]
#[case(Capabilities::from(Capability::RetransmissionAck) | Capability::RetransmissionNack)]
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

    let alice_metrics = Arc::new(SessionTelemetry::new(id, alice_cfg));

    let bob_metrics = Arc::new(SessionTelemetry::new(id, bob_cfg));

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
        None,
        alice_metrics.clone(),
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
        None,
        bob_metrics.clone(),
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

    // Verify metrics updated correctly
    let snapshot = alice_metrics.snapshot();

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

    let cap_suffix = if cap.is_empty() {
        "plain"
    } else if cap.contains(Capability::RetransmissionAck | Capability::RetransmissionNack) {
        "both"
    } else if cap.contains(Capability::RetransmissionAck) {
        "ack"
    } else if cap.contains(Capability::RetransmissionNack) {
        "nack"
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

    let snapshot = bob_metrics.snapshot();

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

    insta::assert_yaml_snapshot!(format!("bob_metrics_tcp_{}", cap_suffix), snapshot, {
        ".snapshot_at" => "[snapshot_ts]",
        ".lifetime.created_at" => "[created_at]",
        ".lifetime.last_activity_at" => "[last_activity_at]",
        ".lifetime.uptime" => "[uptime]",
        ".lifetime.idle" => "[idle]",
        ".session_id.pseudonym" => "[pseudonym]",
    });

    transfer_handle.abort();

    Ok(())
}

#[rstest]
#[case(Capabilities::empty())]
#[case(Capabilities::from(Capability::Segmentation))]
#[tokio::test]
/// Creates paired Hopr sessions with bidirectional communication to prove that
/// data can flow both alice → bob and bob → alice using SURB-enabled routing.
async fn bidirectional_tcp_session(#[case] cap: Capabilities) -> anyhow::Result<()> {
    const MSG_LEN: usize = 4096;

    let start_time = SystemTime::now();

    let dst: Address = (&ChainKeypair::random()).into();
    let pseudonym = HoprPseudonym::random();
    let id = SessionId::new(1u64, pseudonym);
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

    let alice_metrics = Arc::new(SessionTelemetry::new(id, alice_cfg));

    let bob_metrics = Arc::new(SessionTelemetry::new(id, bob_cfg));

    // Alice uses Forward with return_options to enable SURB production
    let mut alice_session = HoprSession::new(
        id,
        DestinationRouting::Forward {
            destination: Box::new(dst.into()),
            pseudonym: Some(pseudonym),
            forward_options: RoutingOptions::Hops(0_u32.try_into()?),
            return_options: Some(RoutingOptions::Hops(0_u32.try_into()?)),
        },
        alice_cfg,
        (
            alice_tx,
            alice_rx.map(|(_, d)| ApplicationDataIn {
                data: d.data,
                packet_info: Default::default(),
            }),
        ),
        None,
        alice_metrics.clone(),
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
        None,
        bob_metrics.clone(),
    )?;

    // Alice sends to Bob
    let alice_msg: [u8; MSG_LEN] = hopr_crypto_random::random_bytes();
    alice_session.write_all(&alice_msg).await?;
    alice_session.flush().await?;

    let mut recv_from_alice = [0u8; MSG_LEN];
    bob_session.read_exact(&mut recv_from_alice).await?;
    assert_eq!(recv_from_alice, alice_msg);

    // Bob sends to Alice
    let bob_msg: [u8; MSG_LEN] = hopr_crypto_random::random_bytes();
    bob_session.write_all(&bob_msg).await?;
    bob_session.flush().await?;

    let mut recv_from_bob = [0u8; MSG_LEN];
    alice_session.read_exact(&mut recv_from_bob).await?;
    assert_eq!(recv_from_bob, bob_msg);

    // Verify alice metrics - should have bytes_in (from bob) and bytes_out (to bob)
    let alice_snapshot = alice_metrics.snapshot();

    assert!(alice_snapshot.snapshot_at >= start_time, "snapshot_at should be recent");

    let cap_suffix = if cap.is_empty() { "plain" } else { "seg" };
    insta::assert_yaml_snapshot!(format!("bidirectional_alice_{}", cap_suffix), alice_snapshot, {
        ".snapshot_at" => "[snapshot_ts]",
        ".lifetime.created_at" => "[created_at]",
        ".lifetime.last_activity_at" => "[last_activity_at]",
        ".lifetime.uptime" => "[uptime]",
        ".lifetime.idle" => "[idle]",
        ".session_id.pseudonym" => "[pseudonym]",
    });

    // Verify bob metrics - should have bytes_in (from alice) and bytes_out (to alice)
    let bob_snapshot = bob_metrics.snapshot();

    assert!(bob_snapshot.snapshot_at >= start_time, "snapshot_at should be recent");

    insta::assert_yaml_snapshot!(format!("bidirectional_bob_{}", cap_suffix), bob_snapshot, {
        ".snapshot_at" => "[snapshot_ts]",
        ".lifetime.created_at" => "[created_at]",
        ".lifetime.last_activity_at" => "[last_activity_at]",
        ".lifetime.uptime" => "[uptime]",
        ".lifetime.idle" => "[idle]",
        ".session_id.pseudonym" => "[pseudonym]",
    });

    Ok(())
}

#[tokio::test]
/// Tests that SURB metrics are correctly captured when using set_surb_estimator().
async fn surb_metrics_tracking() -> anyhow::Result<()> {
    use std::sync::atomic::Ordering;

    let id = SessionId::new(1u64, HoprPseudonym::random());
    let metrics = Arc::new(SessionTelemetry::new(id, Default::default()));

    // Create a SURB estimator and set non-zero values
    let surb_estimator = AtomicSurbFlowEstimator::default();
    surb_estimator.produced.store(100, Ordering::Relaxed);
    surb_estimator.consumed.store(40, Ordering::Relaxed);

    // Set the estimator with target buffer
    let state = BalancerStateData::new(Default::default());
    state.target_surb_buffer_size.store(200, Ordering::Relaxed);
    metrics.set_balancer_data(surb_estimator, state.into());

    // Take snapshot - SURB values are non zero
    let snapshot = metrics.snapshot();

    insta::assert_yaml_snapshot!("surb_metrics", snapshot, {
        ".snapshot_at" => "[snapshot_ts]",
        ".lifetime.created_at" => "[created_at]",
        ".lifetime.last_activity_at" => "[last_activity_at]",
        ".lifetime.uptime" => "[uptime]",
        ".lifetime.idle" => "[idle]",
        ".session_id.pseudonym" => "[pseudonym]",
        ".surb.rate_per_sec" => "[rate_per_sec]",
    });

    Ok(())
}

#[rstest]
#[case(Capabilities::from(Capability::Segmentation))]
#[tokio::test]
/// Tests that frame buffer metrics are correctly captured when segmentation is enabled
/// and enough data is sent to create multiple frames.
async fn frame_buffer_metrics(#[case] cap: Capabilities) -> anyhow::Result<()> {
    const FRAME_MTU: usize = 1500;
    const NUM_FRAMES: usize = 5;
    const MSG_LEN: usize = FRAME_MTU * NUM_FRAMES;

    let start_time = SystemTime::now();

    let dst: Address = (&ChainKeypair::random()).into();
    let id = SessionId::new(1u64, HoprPseudonym::random());
    let (alice_tx, bob_rx) = futures::channel::mpsc::unbounded::<(DestinationRouting, ApplicationDataOut)>();
    let (bob_tx, alice_rx) = futures::channel::mpsc::unbounded::<(DestinationRouting, ApplicationDataOut)>();

    let alice_cfg = HoprSessionConfig {
        capabilities: cap,
        frame_mtu: FRAME_MTU,
        ..Default::default()
    };
    let bob_cfg = HoprSessionConfig {
        capabilities: cap,
        frame_mtu: FRAME_MTU,
        ..Default::default()
    };

    let alice_metrics = Arc::new(SessionTelemetry::new(id, alice_cfg));

    let bob_metrics = Arc::new(SessionTelemetry::new(id, bob_cfg));

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
        None,
        alice_metrics.clone(),
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
        None,
        bob_metrics.clone(),
    )?;

    // Send enough data to create multiple frames
    let msg: [u8; MSG_LEN] = hopr_crypto_random::random_bytes();
    alice_session.write_all(&msg).await?;
    alice_session.flush().await?;

    let mut recv_msg = [0u8; MSG_LEN];
    bob_session.read_exact(&mut recv_msg).await?;

    assert_eq!(recv_msg, msg);

    // Verify alice metrics - frames_emitted should be non-zero (sender)
    let alice_snapshot = alice_metrics.snapshot();

    assert!(alice_snapshot.snapshot_at >= start_time, "snapshot_at should be recent");

    insta::assert_yaml_snapshot!("frame_buffer_alice", alice_snapshot, {
        ".snapshot_at" => "[snapshot_ts]",
        ".lifetime.created_at" => "[created_at]",
        ".lifetime.last_activity_at" => "[last_activity_at]",
        ".lifetime.uptime" => "[uptime]",
        ".lifetime.idle" => "[idle]",
        ".session_id.pseudonym" => "[pseudonym]",
    });

    // Verify bob metrics - frames_completed should be non-zero (receiver)
    let bob_snapshot = bob_metrics.snapshot();

    assert!(bob_snapshot.snapshot_at >= start_time, "snapshot_at should be recent");

    insta::assert_yaml_snapshot!("frame_buffer_bob", bob_snapshot, {
        ".snapshot_at" => "[snapshot_ts]",
        ".lifetime.created_at" => "[created_at]",
        ".lifetime.last_activity_at" => "[last_activity_at]",
        ".lifetime.uptime" => "[uptime]",
        ".lifetime.idle" => "[idle]",
        ".session_id.pseudonym" => "[pseudonym]",
    });

    Ok(())
}
