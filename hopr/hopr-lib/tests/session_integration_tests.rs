use futures::StreamExt;
use hopr_api::types::crypto_random::Randomizable;
use hopr_lib::{
    api::types::{
        crypto::{keypairs::Keypair, prelude::ChainKeypair},
        internal::{
            prelude::HoprPseudonym,
            routing::{DestinationRouting, RoutingOptions},
        },
        primitive::prelude::Address,
    },
    exports::{
        network::types::udp::{ConnectedUdpStream, UdpStreamParallelism},
        transport::{
            ApplicationDataIn, ApplicationDataOut,
            session::{Capabilities, Capability, HoprSession, HoprSessionConfig, transfer_session},
        },
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
    let id = HoprPseudonym::random();
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
        DestinationRouting::Return(id.into()),
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
    let id = HoprPseudonym::random();
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
        DestinationRouting::Return(id.into()),
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
    let id = pseudonym;
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
        DestinationRouting::Return(id.into()),
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

/// Bridges a paired session to a `ConnectedUdpStream` with `transfer_session` (exactly as
/// `hopr-session-server-forwarder` does on the exit node), sends a single UDP datagram of
/// `datagram_len` bytes into it, and returns how many bytes the client side receives from a
/// SINGLE `read` — i.e. how much of the datagram is delivered with its boundary intact.
///
/// A datagram-oriented consumer (the WireGuard pump/neptun) does one read per datagram and treats
/// the returned bytes as one datagram; the datagram-preserving contract is therefore
/// "one read == the whole datagram".
async fn single_read_of_one_forwarded_udp_datagram(datagram_len: usize) -> anyhow::Result<usize> {
    const BUF_LEN: usize = 16384; // HOPR_UDP_BUFFER_SIZE used by the exit forwarder

    let dst: Address = (&ChainKeypair::random()).into();
    let id = HoprPseudonym::random();
    let (alice_tx, bob_rx) = futures::channel::mpsc::unbounded::<(DestinationRouting, ApplicationDataOut)>();
    let (bob_tx, alice_rx) = futures::channel::mpsc::unbounded::<(DestinationRouting, ApplicationDataOut)>();

    // The WireGuard session uses Segmentation + NoDelay; frame_mtu defaults to 1500.
    let cfg = HoprSessionConfig {
        capabilities: Capabilities::from(Capability::Segmentation) | Capability::NoDelay,
        ..Default::default()
    };
    assert_eq!(
        cfg.frame_mtu, 1500,
        "test assumes the WireGuard session frame_mtu of 1500"
    );

    // Exit-side session endpoint (bridged to the WireGuard server).
    let mut alice_session = HoprSession::new(
        id,
        DestinationRouting::forward_only(dst, RoutingOptions::Hops(0_u32.try_into()?)),
        cfg.clone(),
        (
            alice_tx,
            alice_rx.map(|(_, d)| ApplicationDataIn {
                data: d.data,
                packet_info: Default::default(),
            }),
        ),
        None,
    )?;

    // Client-side session endpoint (where the WireGuard pump reads).
    let mut bob_session = HoprSession::new(
        id,
        DestinationRouting::Return(id.into()),
        cfg,
        (
            bob_tx,
            bob_rx.map(|(_, d)| ApplicationDataIn {
                data: d.data,
                packet_info: Default::default(),
            }),
        ),
        None,
    )?;

    let mut udp_bridge = ConnectedUdpStream::builder()
        .with_buffer_size(BUF_LEN)
        .with_queue_size(512)
        .with_receiver_parallelism(UdpStreamParallelism::Auto)
        .build(("127.0.0.1", 0))?;
    let addr = *udp_bridge.bound_address();

    let (ready_tx, ready_rx) = oneshot::channel();
    let transfer_handle = tokio::task::spawn(async move {
        ready_tx.send(()).ok();
        transfer_session(&mut alice_session, &mut udp_bridge, BUF_LEN, None).await
    });
    ready_rx.await.ok();

    // A single UDP datagram from the WireGuard server. When it exceeds frame_mtu it models a
    // UDP-GSO super-buffer (several WireGuard packets coalesced into one UDP send), which is what
    // the exit receives over the high-MTU/loopback path to a co-located WireGuard server.
    let datagram: Vec<u8> = (0..datagram_len).map(|i| (i % 251) as u8).collect();
    let sender = UdpSocket::bind(("127.0.0.1", 0)).await?;
    let sent = sender.send_to(&datagram, addr).await?;
    assert_eq!(sent, datagram_len);

    // The WireGuard pump does ONE read per datagram and hands the result to neptun.
    let mut buf = vec![0u8; datagram_len];
    let n = tokio::time::timeout(std::time::Duration::from_secs(5), bob_session.read(&mut buf)).await??;
    assert_eq!(
        &buf[..n],
        &datagram[..n],
        "delivered bytes must be a prefix of the sent datagram"
    );

    transfer_handle.abort();
    Ok(n)
}

#[tokio::test]
/// Reproduces hoprnet#8356: a UDP datagram larger than `frame_mtu` (a WireGuard UDP-GSO
/// super-buffer) forwarded through `transfer_session` into the byte-stream session is split at
/// `frame_mtu`. The client's single `read` therefore returns only one frame (<= 1500) instead of
/// the whole datagram, so neptun sees a partial/misaligned buffer (`InvalidPacket`/`InvalidAeadTag`)
/// and, after a burst, trips the `DecapStalled` guard and reconnects.
///
/// The session must preserve UDP datagram boundaries for UDP targets so that one datagram is
/// delivered to the peer per read.
async fn udp_datagram_larger_than_frame_mtu_loses_boundary() -> anyhow::Result<()> {
    const DATAGRAM_LEN: usize = 2904; // two 1452-byte WireGuard packets coalesced by UDP GSO

    let n = single_read_of_one_forwarded_udp_datagram(DATAGRAM_LEN).await?;

    assert_eq!(
        n, DATAGRAM_LEN,
        "datagram boundary lost: a {DATAGRAM_LEN}-byte UDP datagram was split by the frame_mtu=1500 session; the \
         client read only {n} bytes (one frame) instead of the whole datagram"
    );
    Ok(())
}

#[tokio::test]
/// Control for #8356: a datagram at/below `frame_mtu` is delivered whole in a single read, so only
/// oversized (GSO-coalesced) datagrams trigger the field failure.
async fn udp_datagram_within_frame_mtu_preserves_boundary() -> anyhow::Result<()> {
    const DATAGRAM_LEN: usize = 1452; // a single WireGuard packet

    let n = single_read_of_one_forwarded_udp_datagram(DATAGRAM_LEN).await?;

    assert_eq!(
        n, DATAGRAM_LEN,
        "a <= frame_mtu datagram must arrive whole in one read; got {n}"
    );
    Ok(())
}
