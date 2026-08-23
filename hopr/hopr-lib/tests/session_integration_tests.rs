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

/// Wires a paired session to a `ConnectedUdpStream` via `transfer_session` (exactly as
/// `hopr-session-server-forwarder` does on the exit node) and returns the client-side session plus
/// the UDP address datagrams should be sent to. The session requests `Datagram` (the #8356 fix), so
/// it preserves UDP datagram boundaries: one frame per write, one datagram per read. The forwarding
/// task is detached and cancelled when the test runtime shuts down.
async fn datagram_udp_bridge() -> anyhow::Result<(HoprSession, std::net::SocketAddr)> {
    const BUF_LEN: usize = 16384; // HOPR_UDP_BUFFER_SIZE used by the exit forwarder

    let dst: Address = (&ChainKeypair::random()).into();
    let id = HoprPseudonym::random();
    let (alice_tx, bob_rx) = futures::channel::mpsc::unbounded::<(DestinationRouting, ApplicationDataOut)>();
    let (bob_tx, alice_rx) = futures::channel::mpsc::unbounded::<(DestinationRouting, ApplicationDataOut)>();

    // The WireGuard session uses Segmentation + NoDelay; on a stateless session NoDelay enables
    // datagram-boundary preservation (UDP-like framing). frame_mtu defaults to 1500.
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
    let bob_session = HoprSession::new(
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
    tokio::task::spawn(async move {
        ready_tx.send(()).ok();
        transfer_session(&mut alice_session, &mut udp_bridge, BUF_LEN, None).await
    });
    ready_rx.await.ok();

    Ok((bob_session, addr))
}

/// Sends one UDP datagram of `datagram_len` bytes through the bridge and returns how many bytes the
/// client receives from a SINGLE `read` — i.e. how much of the datagram is delivered with its
/// boundary intact. A datagram-oriented consumer (the WireGuard pump/neptun) does one read per
/// datagram, so the datagram-preserving contract is "one read == the whole datagram". A
/// `datagram_len` above frame_mtu models a UDP-GSO super-buffer (several WireGuard packets in one
/// UDP send), which is what the exit receives over the high-MTU path to a co-located WG server.
async fn single_read_of_one_forwarded_udp_datagram(datagram_len: usize) -> anyhow::Result<usize> {
    let (mut bob_session, addr) = datagram_udp_bridge().await?;

    let datagram: Vec<u8> = (0..datagram_len).map(|i| (i % 251) as u8).collect();
    let sender = UdpSocket::bind(("127.0.0.1", 0)).await?;
    assert_eq!(sender.send_to(&datagram, addr).await?, datagram_len);

    // Over-size the read buffer so a coalescing regression (more than one datagram in a single
    // read) surfaces as `n > datagram_len` for the caller to assert on, instead of being silently
    // truncated to `datagram_len` by an exact-size buffer.
    let mut buf = vec![0u8; datagram_len + 8192];
    let n = tokio::time::timeout(std::time::Duration::from_secs(5), bob_session.read(&mut buf)).await??;
    let checked = n.min(datagram_len);
    assert_eq!(
        &buf[..checked],
        &datagram[..checked],
        "delivered bytes must be a prefix of the sent datagram"
    );
    Ok(n)
}

#[rstest]
#[case::larger_than_frame_mtu(2904)] // two 1452-byte WireGuard packets coalesced by UDP GSO
#[case::within_frame_mtu(1452)] // a single WireGuard packet
#[tokio::test]
/// Regression test for hoprnet#8356. A UDP datagram forwarded through `transfer_session` must be
/// delivered to the peer as a single `read`. Before the fix the byte-stream session split a
/// datagram larger than `frame_mtu` across frames, so the client's single `read` returned only one
/// frame (<= 1500) and neptun rejected the partial buffer (`InvalidPacket`/`InvalidAeadTag`) until
/// the `DecapStalled` guard reconnected. With the `Datagram` capability the session preserves the
/// boundary (one frame per write), whatever the datagram size.
async fn udp_datagram_is_delivered_whole(#[case] datagram_len: usize) -> anyhow::Result<()> {
    let n = single_read_of_one_forwarded_udp_datagram(datagram_len).await?;

    assert_eq!(
        n, datagram_len,
        "datagram boundary not preserved: a {datagram_len}-byte UDP datagram was delivered in a {n}-byte read"
    );
    Ok(())
}

#[tokio::test]
/// #8356: back-to-back datagrams of mixed sizes each arrive whole, in order, one per read — the
/// datagram session must neither split a datagram nor coalesce adjacent ones.
async fn udp_back_to_back_datagrams_each_arrive_whole_in_order() -> anyhow::Result<()> {
    const BUF_LEN: usize = 16384;
    // Mixed sizes incl. > frame_mtu; distinct fill bytes to check ordering and boundaries.
    let sizes: [usize; 4] = [1452, 2904, 900, 4308];

    let (mut bob_session, addr) = datagram_udp_bridge().await?;

    let sender = UdpSocket::bind(("127.0.0.1", 0)).await?;
    for (i, &len) in sizes.iter().enumerate() {
        assert_eq!(sender.send_to(&vec![i as u8; len], addr).await?, len);
    }

    for (i, &len) in sizes.iter().enumerate() {
        let mut buf = vec![0u8; BUF_LEN];
        let n = tokio::time::timeout(std::time::Duration::from_secs(5), bob_session.read(&mut buf)).await??;
        assert_eq!(n, len, "datagram {i} ({len} B) arrived as a {n}-byte read");
        assert!(
            buf[..n].iter().all(|&b| b == i as u8),
            "datagram {i} content/order mismatch"
        );
    }

    Ok(())
}

#[rstest]
#[case(Capability::RetransmissionAck)]
#[case(Capability::RetransmissionNack)]
#[tokio::test]
/// Datagram-boundary preservation is stateless-only. On a reliable (retransmitting) session,
/// NoDelay keeps only its buffering behavior — a write larger than frame_mtu is still split across
/// frames (byte-stream framing), so a single read returns at most one frame and the payload is NOT
/// delivered whole. This is what keeps oversized datagram frames off the reliable socket, whose
/// NACK missing-segment bitmap cannot address their segments.
async fn nodelay_on_reliable_session_keeps_byte_stream_framing(
    #[case] retransmission: Capability,
) -> anyhow::Result<()> {
    let id = HoprPseudonym::random();
    let dst: Address = (&ChainKeypair::random()).into();
    let (alice_tx, bob_rx) = futures::channel::mpsc::unbounded::<(DestinationRouting, ApplicationDataOut)>();
    let (bob_tx, alice_rx) = futures::channel::mpsc::unbounded::<(DestinationRouting, ApplicationDataOut)>();

    // Reliable session (selects the stateful socket) that also requests NoDelay.
    let cfg = HoprSessionConfig {
        capabilities: Capabilities::from(Capability::Segmentation) | Capability::NoDelay | retransmission,
        ..Default::default()
    };

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

    // A payload well above frame_mtu (1500): if datagram mode were (wrongly) active it would arrive
    // in one read; on a reliable socket it must be split at frame_mtu instead.
    let payload = vec![0x5Au8; 2904];
    alice_session.write_all(&payload).await?;
    alice_session.flush().await?;

    let mut buf = vec![0u8; payload.len()];
    let n = tokio::time::timeout(std::time::Duration::from_secs(5), bob_session.read(&mut buf)).await??;
    assert!(
        n < payload.len(),
        "reliable NoDelay session must keep byte-stream framing (split at frame_mtu), but a single read returned all \
         {n} bytes — datagram boundaries were preserved on a reliable socket"
    );
    Ok(())
}
