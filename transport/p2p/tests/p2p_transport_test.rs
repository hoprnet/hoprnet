use std::{
    net::{Ipv4Addr, SocketAddrV4, TcpListener},
    str::FromStr,
};

use anyhow::Context;
use futures::{
    SinkExt, StreamExt,
    channel::mpsc::{Receiver, Sender},
};
use hopr_crypto_types::{keypairs::Keypair, prelude::OffchainKeypair};
use hopr_platform::time::native::current_time;
use hopr_transport_p2p::{HoprLibp2pNetworkBuilder, HoprNetwork};
use hopr_transport_probe::ping::PingQueryReplier;
use hopr_transport_protocol::PeerDiscovery;
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
    pub send_msg: Sender<(PeerId, Box<[u8]>)>,
    pub recv_msg: Receiver<(PeerId, Box<[u8]>)>,
}
#[allow(clippy::upper_case_acronyms)]
pub(crate) enum Announcement {
    QUIC,
}

pub(crate) type TestSwarm = HoprNetwork;

async fn build_p2p_swarm(
    announcement: Announcement,
) -> anyhow::Result<(Interface, (TestSwarm, impl std::future::Future<Output = ()>))> {
    let random_port = random_free_local_ipv4_port().context("could not find a free port")?;
    let random_keypair = OffchainKeypair::random();
    let identity: libp2p::identity::Keypair = (&random_keypair).into();
    let peer_id: PeerId = identity.public().into();

    let (transport_updates_tx, transport_updates_rx) = futures::channel::mpsc::unbounded::<PeerDiscovery>();
    let (heartbeat_requests_tx, _heartbeat_requests_rx) =
        futures::channel::mpsc::unbounded::<(PeerId, PingQueryReplier)>();

    let multiaddress = match announcement {
        Announcement::QUIC => format!("/ip4/127.0.0.1/udp/{random_port}/quic-v1"),
    };
    let multiaddress = Multiaddr::from_str(&multiaddress).context("failed to create a valid multiaddress")?;

    let swarm = HoprLibp2pNetworkBuilder::new(identity, transport_updates_rx, vec![multiaddress.clone()]).await;
    let (network, process) =
        swarm.into_network_with_stream_protocol_process(hopr_transport_protocol::CURRENT_HOPR_MSG_PROTOCOL, true);

    let msg_codec = hopr_transport_protocol::HoprBinaryCodec {};
    let network_for_stats = network.clone();
    let (wire_msg_tx, wire_msg_rx) =
        hopr_transport_protocol::stream::process_stream_protocol(msg_codec, network.clone(), move |peer| {
            network_for_stats.get_packet_stats(peer)
        })
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
    pub static ref RANDOM_GIBBERISH: Box<[u8]> =
        Box::from(hopr_crypto_random::random_bytes::<TRANSPORT_PAYLOAD_SIZE>());
}

pub fn generate_packets_of_hopr_payload_size(count: usize) -> Vec<Box<[u8]>> {
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
use hopr_transport_network::observation::PeerPacketStatsSnapshot;
use libp2p::{Multiaddr, PeerId};
use more_asserts::assert_gt;
use tokio::{
    task::{JoinHandle, spawn},
    time::{sleep, timeout},
};

#[ignore]
#[tokio::test]
async fn p2p_only_communication_quic() -> anyhow::Result<()> {
    let (mut api1, (_swarm1, process1)) = build_p2p_swarm(Announcement::QUIC).await?;
    let (api2, (_swarm2, process2)) = build_p2p_swarm(Announcement::QUIC).await?;

    let _sjh1 = SelfClosingJoinHandle::new(process1);
    let _sjh2 = SelfClosingJoinHandle::new(process2);

    // Announce nodes to each other
    api1.update_from_announcements
        .unbounded_send(PeerDiscovery::Announce(api2.me, vec![api2.address.clone()]))
        .context("failed to send announcement")?;
    api2.update_from_announcements
        .unbounded_send(PeerDiscovery::Announce(api1.me, vec![api1.address.clone()]))
        .context("failed to send announcement")?;

    // Wait for node listen_on and announcements
    sleep(std::time::Duration::from_secs(3)).await;

    let start = current_time();

    // ~10MB of data
    let packet_count: usize = 2 * 1024 * 10;
    for _ in 0..packet_count {
        api1.send_msg
            .send((api2.me, RANDOM_GIBBERISH.clone()))
            .await
            .context("failed to send message")?;
    }

    timeout(
        std::time::Duration::from_secs(30),
        api2.recv_msg.take(packet_count).collect::<Vec<_>>(),
    )
    .await?;

    let speed_in_mbytes_s =
        (RANDOM_GIBBERISH.len() * packet_count) as f64 / (start.elapsed()?.as_millis() as f64 * 1000f64);

    assert_gt!(
        speed_in_mbytes_s,
        100.0f64,
        "The measured speed for data transfer is ~{}MB/s",
        speed_in_mbytes_s
    );

    Ok(())
}

#[ignore]
#[tokio::test]
async fn p2p_peer_packet_stats_quic() -> anyhow::Result<()> {
    let (mut api1, (swarm1, process1)) = build_p2p_swarm(Announcement::QUIC).await?;
    let (mut api2, (swarm2, process2)) = build_p2p_swarm(Announcement::QUIC).await?;

    let _sjh1 = SelfClosingJoinHandle::new(process1);
    let _sjh2 = SelfClosingJoinHandle::new(process2);

    // Announce nodes to each other
    api1.update_from_announcements
        .unbounded_send(PeerDiscovery::Announce(api2.me, vec![api2.address.clone()]))
        .context("failed to send announcement")?;
    api2.update_from_announcements
        .unbounded_send(PeerDiscovery::Announce(api1.me, vec![api1.address.clone()]))
        .context("failed to send announcement")?;

    // Wait until both peers are tracked in each other's NetworkPeerTracker.
    // The discovery behavior dials with an initial backoff of ~3s (with jitter),
    // so a fixed sleep isn't reliable. Poll until the peer entry exists, which
    // guarantees that get_packet_stats() will return Some when the stream opens.
    timeout(std::time::Duration::from_secs(30), async {
        loop {
            if swarm1.packet_stats_snapshot(&api2.me).is_some() && swarm2.packet_stats_snapshot(&api1.me).is_some() {
                break;
            }
            sleep(std::time::Duration::from_millis(100)).await;
        }
    })
    .await
    .context("timed out waiting for peers to be tracked")?;

    // -- Phase 1: peer1 → peer2 (5 packets) --
    let phase1_count: usize = 5;
    for _ in 0..phase1_count {
        api1.send_msg
            .send((api2.me, RANDOM_GIBBERISH.clone()))
            .await
            .context("failed to send message from peer1")?;
    }

    timeout(
        std::time::Duration::from_secs(10),
        api2.recv_msg.by_ref().take(phase1_count).collect::<Vec<_>>(),
    )
    .await
    .context("timed out waiting for peer2 to receive phase 1 packets")?;

    let stats1_for_peer2 = swarm1
        .packet_stats_snapshot(&api2.me)
        .expect("swarm1 should have stats for peer2");
    assert_eq!(
        stats1_for_peer2,
        PeerPacketStatsSnapshot {
            packets_out: phase1_count as u64,
            bytes_out: (phase1_count * TRANSPORT_PAYLOAD_SIZE) as u64,
            packets_in: 0,
            bytes_in: 0,
        },
        "swarm1 stats after phase 1"
    );

    let stats2_for_peer1 = swarm2
        .packet_stats_snapshot(&api1.me)
        .expect("swarm2 should have stats for peer1");
    assert_eq!(
        stats2_for_peer1,
        PeerPacketStatsSnapshot {
            packets_out: 0,
            bytes_out: 0,
            packets_in: phase1_count as u64,
            bytes_in: (phase1_count * TRANSPORT_PAYLOAD_SIZE) as u64,
        },
        "swarm2 stats after phase 1"
    );

    // -- Phase 2: peer2 → peer1 (3 packets) --
    let phase2_count: usize = 3;
    for _ in 0..phase2_count {
        api2.send_msg
            .send((api1.me, RANDOM_GIBBERISH.clone()))
            .await
            .context("failed to send message from peer2")?;
    }

    timeout(
        std::time::Duration::from_secs(10),
        api1.recv_msg.by_ref().take(phase2_count).collect::<Vec<_>>(),
    )
    .await
    .context("timed out waiting for peer1 to receive phase 2 packets")?;

    let stats1_for_peer2 = swarm1
        .packet_stats_snapshot(&api2.me)
        .expect("swarm1 should have stats for peer2 after phase 2");
    assert_eq!(
        stats1_for_peer2,
        PeerPacketStatsSnapshot {
            packets_out: phase1_count as u64,
            bytes_out: (phase1_count * TRANSPORT_PAYLOAD_SIZE) as u64,
            packets_in: phase2_count as u64,
            bytes_in: (phase2_count * TRANSPORT_PAYLOAD_SIZE) as u64,
        },
        "swarm1 stats after phase 2"
    );

    let stats2_for_peer1 = swarm2
        .packet_stats_snapshot(&api1.me)
        .expect("swarm2 should have stats for peer1 after phase 2");
    assert_eq!(
        stats2_for_peer1,
        PeerPacketStatsSnapshot {
            packets_out: phase2_count as u64,
            bytes_out: (phase2_count * TRANSPORT_PAYLOAD_SIZE) as u64,
            packets_in: phase1_count as u64,
            bytes_in: (phase1_count * TRANSPORT_PAYLOAD_SIZE) as u64,
        },
        "swarm2 stats after phase 2"
    );

    // -- Phase 3: all_packet_stats bulk API --
    let all_stats1 = swarm1.all_packet_stats();
    assert_eq!(all_stats1.len(), 1, "swarm1 should track exactly 1 remote peer");
    assert_eq!(all_stats1[0].0, api2.me, "swarm1 should track peer2");
    assert_eq!(
        all_stats1[0].1, stats1_for_peer2,
        "swarm1 bulk stats should match per-peer snapshot"
    );

    let all_stats2 = swarm2.all_packet_stats();
    assert_eq!(all_stats2.len(), 1, "swarm2 should track exactly 1 remote peer");
    assert_eq!(all_stats2[0].0, api1.me, "swarm2 should track peer1");
    assert_eq!(
        all_stats2[0].1, stats2_for_peer1,
        "swarm2 bulk stats should match per-peer snapshot"
    );

    Ok(())
}
