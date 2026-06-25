// This integration test requires the `runtime-tokio` and `transport-quic` features.
#![cfg(all(feature = "runtime-tokio", feature = "transport-quic"))]

use std::{
    net::{Ipv4Addr, SocketAddrV4, TcpListener},
    str::FromStr,
};

use anyhow::Context;
use bytes::Bytes;
use futures::{
    SinkExt, StreamExt,
    channel::mpsc::{Receiver, Sender},
};
use hopr_api::types::crypto::{keypairs::Keypair, prelude::OffchainKeypair};
use hopr_transport_p2p::{HoprLibp2pNetworkBuilder, HoprNetwork, PeerDiscovery};
use hopr_transport_probe::ping::PingQueryReplier;
use lazy_static::lazy_static;

pub fn random_free_local_ipv4_port() -> Option<u16> {
    let socket = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0);
    TcpListener::bind(socket)
        .and_then(|listener| listener.local_addr())
        .map(|addr| addr.port())
        .ok()
}

pub(crate) struct Interface {
    pub me: PeerId,
    pub address: Multiaddr,
    pub update_from_announcements: futures::channel::mpsc::UnboundedSender<PeerDiscovery>,
    #[allow(dead_code)]
    pub send_heartbeat: futures::channel::mpsc::UnboundedSender<(PeerId, PingQueryReplier)>,
    // ---
    pub send_msg: Sender<(PeerId, Bytes)>,
    pub recv_msg: Receiver<(PeerId, Bytes)>,
}
#[allow(clippy::upper_case_acronyms)]
pub(crate) enum Announcement {
    QUIC,
}

pub(crate) type TestSwarm = HoprNetwork;

async fn build_p2p_swarm(
    announcement: Announcement,
) -> anyhow::Result<(Interface, (TestSwarm, hopr_api::network::BoxedProcessFn))> {
    let random_port = random_free_local_ipv4_port().context("could not find a free port")?;
    let random_keypair = OffchainKeypair::random();
    let peer_id: PeerId = libp2p::identity::Keypair::from(&random_keypair).public().into();

    let (transport_updates_tx, transport_updates_rx) = futures::channel::mpsc::unbounded::<PeerDiscovery>();
    let (heartbeat_requests_tx, _heartbeat_requests_rx) =
        futures::channel::mpsc::unbounded::<(PeerId, PingQueryReplier)>();

    let multiaddress = match announcement {
        Announcement::QUIC => format!("/ip4/127.0.0.1/udp/{random_port}/quic-v1"),
    };
    let multiaddress = Multiaddr::from_str(&multiaddress).context("failed to create a valid multiaddress")?;

    let network_builder = HoprLibp2pNetworkBuilder::new(transport_updates_rx);
    let (network, process) = network_builder
        .build(
            &random_keypair,
            vec![multiaddress.clone()],
            hopr_transport::protocol::CURRENT_HOPR_MSG_PROTOCOL,
            true,
        )
        .await
        .map_err(|e| anyhow::anyhow!("failed to build network: {e}"))?;

    let msg_codec = hopr_transport::protocol::HoprBinaryCodec {};
    let (wire_msg_tx, wire_msg_rx) =
        hopr_transport::protocol::stream::process_stream_protocol(msg_codec, network.clone(), Default::default())
            .await?;

    let api = Interface {
        me: peer_id,
        address: multiaddress,
        update_from_announcements: transport_updates_tx,
        send_heartbeat: heartbeat_requests_tx,
        send_msg: wire_msg_tx,
        recv_msg: wire_msg_rx,
    };

    Ok((api, (network, process)))
}

const TRANSPORT_PAYLOAD_SIZE: usize = HoprPacket::SIZE;

lazy_static! {
    pub static ref RANDOM_GIBBERISH: Bytes =
        Bytes::copy_from_slice(&hopr_api::types::crypto_random::random_bytes::<TRANSPORT_PAYLOAD_SIZE>());
}

pub fn generate_packets_of_hopr_payload_size(count: usize) -> Vec<Bytes> {
    let mut packets = Vec::with_capacity(count);
    for _ in 0..count {
        packets.push(RANDOM_GIBBERISH.clone());
    }
    packets
}

pub struct SelfClosingJoinHandle {
    handle: Option<JoinHandle<()>>,
}

impl SelfClosingJoinHandle {
    pub fn new<F>(f: F) -> Self
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        Self { handle: Some(spawn(f)) }
    }
}

impl Drop for SelfClosingJoinHandle {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
    }
}

use hopr_crypto_packet::prelude::HoprPacket;
use libp2p::{Multiaddr, PeerId};
use more_asserts::assert_gt;
use tokio::{
    task::{JoinHandle, spawn},
    time::{Instant, sleep, timeout},
};

#[tokio::test(flavor = "multi_thread", worker_threads = 16)]
async fn p2p_only_communication_quic() -> anyhow::Result<()> {
    let (mut api1, (_swarm1, process1)) = build_p2p_swarm(Announcement::QUIC).await?;
    let (mut api2, (_swarm2, process2)) = build_p2p_swarm(Announcement::QUIC).await?;

    let _sjh1 = SelfClosingJoinHandle::new(process1());
    let _sjh2 = SelfClosingJoinHandle::new(process2());

    // Announce nodes to each other
    api1.update_from_announcements
        .unbounded_send(PeerDiscovery::Announce(api2.me, vec![api2.address.clone()]))
        .context("failed to send announcement")?;
    api2.update_from_announcements
        .unbounded_send(PeerDiscovery::Announce(api1.me, vec![api1.address.clone()]))
        .context("failed to send announcement")?;

    // Wait for node listen_on and announcements
    sleep(std::time::Duration::from_secs(3)).await;

    // Pre-prime: send one packet and wait for it on the receiver side so the
    // per-peer QUIC stream is established before the bulk send. This ensures the
    // opening-window ring buffer does not contribute any packet drops to the
    // throughput measurement.
    api1.send_msg
        .send((api2.me, RANDOM_GIBBERISH.clone()))
        .await
        .context("priming send failed")?;
    timeout(std::time::Duration::from_secs(5), api2.recv_msg.next())
        .await
        .context("priming receive timed out")?
        .context("priming receive: channel closed")?;

    // Bulk send: blast all packets into the pre-primed stream. The egress drain
    // is fully non-blocking (drop-oldest ring), so drops are acceptable — we
    // assert on achieved goodput rather than exact delivery.
    let packet_count: usize = 2 * 1024 * 10; // ~10 MB
    let target_bytes = RANDOM_GIBBERISH.len() * packet_count;

    let start = Instant::now();

    let peer = api2.me;
    let mut bulk_sender = api1.send_msg.clone();
    let _sender = SelfClosingJoinHandle::new(async move {
        for _ in 0..packet_count {
            if bulk_sender.send((peer, RANDOM_GIBBERISH.clone())).await.is_err() {
                break;
            }
        }
    });

    // Receive until the target byte count is seen or no packet arrives for 2 s.
    // Measure throughput from `start` to the instant the last packet arrived —
    // not to the end of the loop — so that a handful of ring-buffer drops at the
    // tail do not inflate elapsed with idle timeout time.
    let mut received_bytes = 0usize;
    let mut last_received = start;
    while received_bytes < target_bytes {
        match timeout(std::time::Duration::from_secs(2), api2.recv_msg.next()).await {
            Ok(Some((_, pkt))) => {
                received_bytes += pkt.len();
                last_received = Instant::now();
            }
            _ => break,
        }
    }

    let elapsed = last_received.duration_since(start);
    let speed_in_mbytes_s = received_bytes as f64 / elapsed.as_secs_f64() / 1_000_000.0;

    assert_gt!(
        speed_in_mbytes_s,
        50.0f64,
        "The measured speed for data transfer is ~{speed_in_mbytes_s:.1}MB/s on {received_bytes} bytes received, \
         which is less than the expected 50MB/s",
    );

    Ok(())
}
