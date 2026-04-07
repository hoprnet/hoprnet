use futures::StreamExt;
use hopr_api::types::crypto_random::Randomizable;
use hopr_lib::{
    Address, ApplicationDataIn, ApplicationDataOut, ChainKeypair, ConnectedUdpStream, HoprPseudonym, Keypair,
    UdpStreamParallelism,
    exports::{
        transport::session::{Capabilities, Capability, HoprSession, HoprSessionConfig, SessionId, transfer_session},
        types::internal::routing::{DestinationRouting, RoutingOptions},
    },
};
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
    const BUF_LEN: usize = 16384;
    const MSG_LEN: usize = 9183;

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

    let msg: [u8; MSG_LEN] = hopr_api::types::crypto_random::random_bytes();
    let sender = UdpSocket::bind(("127.0.0.1", 0)).await?;

    let w = sender.send_to(&msg, addr).await?;
    assert_eq!(MSG_LEN, w);

    let mut recv_msg = [0u8; MSG_LEN];
    bob_session.read_exact(&mut recv_msg).await?;

    assert_eq!(recv_msg, msg);
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
    )?;

    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;

    let (ready_tx, ready_rx) = oneshot::channel();
    let transfer_handle = tokio::task::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        ready_tx.send(()).ok();
        transfer_session(&mut alice_session, &mut stream, BUF_LEN, None).await
    });

    let msg: [u8; MSG_LEN] = hopr_api::types::crypto_random::random_bytes();
    let mut sender = TcpStream::connect(addr).await?;

    ready_rx.await.ok();

    sender.write_all(&msg).await?;

    let mut recv_msg = [0u8; MSG_LEN];
    bob_session.read_exact(&mut recv_msg).await?;

    assert_eq!(recv_msg, msg);
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
    )?;

    let alice_msg: [u8; MSG_LEN] = hopr_api::types::crypto_random::random_bytes();
    alice_session.write_all(&alice_msg).await?;
    alice_session.flush().await?;

    let mut recv_from_alice = [0u8; MSG_LEN];
    bob_session.read_exact(&mut recv_from_alice).await?;
    assert_eq!(recv_from_alice, alice_msg);

    let bob_msg: [u8; MSG_LEN] = hopr_api::types::crypto_random::random_bytes();
    bob_session.write_all(&bob_msg).await?;
    bob_session.flush().await?;

    let mut recv_from_bob = [0u8; MSG_LEN];
    alice_session.read_exact(&mut recv_from_bob).await?;
    assert_eq!(recv_from_bob, bob_msg);

    Ok(())
}
